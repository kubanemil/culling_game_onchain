import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import {
  getAssociatedTokenAddressSync,
  getAccount,
  getMint,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { findPDA, getLogs, getVaultMintAddress } from "./helpers";
import { Asset } from "../target/types/asset";
import { expect, assert } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  createNft,
  findMetadataPda,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import {
  percentAmount,
  keypairIdentity,
  createSignerFromKeypair,
  generateSigner,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";

const METAPLEX_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

describe("asset", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.Asset as Program<Asset>;
  const user = provider.publicKey;
  const conn = new web3.Connection(provider.connection.rpcEndpoint, {
    commitment: "confirmed",
  });

  const umi = createUmi(conn);

  const payer = provider.wallet as NodeWallet;
  const creatorWallet = umi.eddsa.createKeypairFromSecretKey(
    new Uint8Array(payer.payer.secretKey)
  );

  const creator = createSignerFromKeypair(umi, creatorWallet);

  umi.use(keypairIdentity(creator));
  umi.use(mplTokenMetadata());

  anchor.setProvider(provider);

  const [vault, mint_address] = await getVaultMintAddress(program, user);
  const cardId = 7;
  const [cardAddress] = findPDA(
    [
      Buffer.from("card"),
      Buffer.from(new Uint8Array([cardId])),
      user.toBuffer(),
    ],
    program.programId
  );

  const vaultAta = await getAssociatedTokenAddressSync(
    mint_address,
    vault,
    true
  );
  const card = generateSigner(umi);

  it("init", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc({
      commitment: "confirmed",
    });

    const logs = await getLogs(program, tx);
    console.log(logs[10]);
  });

  it("Buy tokens", async () => {
    const amount = 20_000_000; // is 1 token, as token::decimals = 6

    const vaultBalanceBefore = await conn.getBalance(vault);
    // invoke instruction
    await program.methods.buyToken(new BN(amount)).rpc({
      commitment: "confirmed",
    });

    // check balance
    const vaultBalanceAfter = await conn.getBalance(vault);
    expect(vaultBalanceAfter - vaultBalanceBefore).equals(1000 * amount);

    // check mint
    const mintAccount = await getMint(conn, mint_address);
    assert(mintAccount.decimals == 6, "Mint has wrong decimals");
    assert(mintAccount.mintAuthority.equals(vault), "Wrong mint authority");
    assert(mintAccount.supply == BigInt(amount), "Mint supply is invalid");

    // check token
    const ata_address = await getAssociatedTokenAddressSync(mint_address, user);
    const ata = await getAccount(conn, ata_address);
    assert(ata.mint.equals(mint_address), "Wrong ATA mint");
    assert(ata.owner.equals(user), "ATA's owner is invalid.");
    assert(ata.amount == BigInt(amount), "ATA's balance is invalid");
  });

  it("buy a card", async () => {
    const cardTokenPrice = 10 * 10 ** 6;

    const tx = await program.methods
      .buyCard(cardId)
      .accounts({
        card: cardAddress,
      })
      .rpc();
    await conn.confirmTransaction(tx, "confirmed");

    // check balance
    const vaultAtaBalance = Number(
      (await conn.getTokenAccountBalance(vaultAta)).value.amount
    );
    expect(vaultAtaBalance).greaterThanOrEqual(
      cardTokenPrice,
      "vault did not receive full amount"
    );

    // check card token
    let tokenAccounts = await conn.getTokenAccountsByOwner(user, {
      programId: TOKEN_PROGRAM_ID,
    });
    for (let account of tokenAccounts.value) {
      let accountInfo = account.account.data;
      console.log(
        `Token Mint address/owner: ${account.pubkey} ${account.account.owner} ${account.account.lamports}`
      );
      console.log(cardAddress.toString(), mint_address.toString());
      // assert(account.pubkey == cardAddress, "Wrong card pubkey")
    }

    const cardInfo = await getMint(conn, cardAddress);
    assert(cardInfo.decimals == 0, "Card is not unit");
    assert(cardInfo.mintAuthority.equals(user), "Wrong owner of the card");
    assert(cardInfo.isInitialized == true, "Mint is not initialized");
  });

  it("init metadata", async () => {
    await createNft(umi, {
      mint: card,
      name: "cullingMetaToken",
      symbol: "cM@",
      uri: "https://arweave.net/culling",
      sellerFeeBasisPoints: percentAmount(5),
      creators: null,
      collectionDetails: {
        __kind: "V1",
        size: 10,
      },
    }).sendAndConfirm(umi);
  });

  it("set metadata", async () => {
    const cardId = 7;
    const name = "cullingMetaToken";
    const uri = "https://mail.google.com/mail/u/0/";

    // const [cardMetadataAddress] = findPDA(
    //   [
    //     Buffer.from("metadata"),
    //     METAPLEX_PROGRAM_ID.toBuffer(),
    //     cardAddress.toBuffer(),
    //   ],
    //   METAPLEX_PROGRAM_ID
    // );
    const cardMetadataAddress = findMetadataPda(umi, {
      mint: cardAddress.toString(),
    });

    const tx = await program.methods
      .setMetadata(cardId, name, uri)
      .accounts({
        signer: user,
        card: cardAddress,
        metadata: cardMetadataAddress,
      })
      .rpc();
  });
});
