import {
  getAssociatedTokenAddressSync,
  getAccount,
  getMint,
} from "@solana/spl-token";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import {
  keypairIdentity,
  createSignerFromKeypair,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  findMetadataPda,
  mplTokenMetadata,
  TokenStandard,
  fetchDigitalAsset,
} from "@metaplex-foundation/mpl-token-metadata";
import { fromWeb3JsPublicKey } from "@metaplex-foundation/umi-web3js-adapters";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { expect, assert } from "chai";
import { findPDA, getVaultMintAddress } from "./helpers";
import { Asset } from "../target/types/asset";

describe("asset", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.Asset as Program<Asset>;
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

  // test variables
  const user = provider.publicKey;
  const cardId = 7;

  const [vault, mint_address] = await getVaultMintAddress(program, user);
  const vaultAta = await getAssociatedTokenAddressSync(
    mint_address,
    vault,
    true
  );
  const [cardAddress] = findPDA(
    [
      Buffer.from("card"),
      Buffer.from(new Uint8Array([cardId])),
      user.toBuffer(),
    ],
    program.programId
  );
  const card = fromWeb3JsPublicKey(cardAddress);

  it("init", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    await conn.confirmTransaction(tx, "confirmed");
  });

  it("buy tokens", async () => {
    const amount = 20_000_000; // is 1 token, as token::decimals = 6

    const vaultBalanceBefore = await conn.getBalance(vault);
    // invoke instruction
    const tx = await program.methods.buyToken(new BN(amount)).rpc();
    await conn.confirmTransaction(tx, "confirmed");

    // check balance
    const vaultBalanceAfter = await conn.getBalance(vault);
    expect(vaultBalanceAfter - vaultBalanceBefore).equals(1000 * amount);

    // check mint
    const mintAccount = await getMint(conn, mint_address);
    assert(mintAccount.decimals == 6, "Mint has wrong decimals");
    assert(mintAccount.mintAuthority.equals(vault), "Wrong mint authority");
    assert(mintAccount.supply == BigInt(amount), "Mint supply is invalid");

    // check ata
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

    // check card
    const cardInfo = await getMint(conn, cardAddress);
    assert(cardInfo.decimals == 0, "Card is not unit");
    assert(cardInfo.mintAuthority.equals(user), "Wrong owner of the card");
    assert(cardInfo.isInitialized == true, "Mint is not initialized");

    // check ata
    const ata_address = await getAssociatedTokenAddressSync(cardAddress, user);
    const ata = await getAccount(conn, ata_address);
    assert(ata.mint.equals(cardAddress), "Wrong ATA mint");
    assert(ata.owner.equals(user), "ATA's owner is invalid.");
    assert(ata.amount == BigInt(1), "ATA's balance is invalid");
  });

  it("create metadata", async () => {
    const [cardMetadataAddress] = findMetadataPda(umi, {
      mint: card,
    });

    const name = "cullingMetaToken";
    const symbol = "cMT";
    const uri = "https://www.culing-game.com/meta/uri/123";

    const tx = await program.methods
      .createMetadata(cardId, name, symbol, uri)
      .accounts({
        signer: user,
        card: cardAddress,
        metadata: cardMetadataAddress,
      })
      .rpc();
    await conn.confirmTransaction(tx, "confirmed");

    // check metadata
    const cardMetadata = await fetchDigitalAsset(umi, card);

    expect(cardMetadata.metadata.name).eq(name, "Invalid card name");
    expect(cardMetadata.metadata.symbol).eq(symbol, "Invalid card symbol");
    expect(cardMetadata.metadata.uri).eq(uri, "Invalid URI");
    expect(cardMetadata.mint.publicKey).eq(card, "Invalid card address");
    expect(cardMetadata.metadata.updateAuthority.toString()).eq(
      vault.toString(),
      "Wrong update authority"
    );
    expect(cardMetadata.metadata.tokenStandard["value"]).eq(
      TokenStandard.FungibleAsset,
      "Invalid token standard"
    );
    expect(cardMetadata.metadata.isMutable).eq(
      true,
      "Card metadata is immutable"
    );
  });

  it("update metadata", async () => {
    const [cardMetadataAddress] = findMetadataPda(umi, {
      mint: card,
    });

    const new_name = "newCullingMetaToken";
    const new_symbol = "ncMT";
    const new_uri = "https://www.culing-game.com/meta/uri/777";

    const tx = await program.methods
      .updateMetadata(cardId, new_name, new_symbol, new_uri)
      .accounts({
        signer: user,
        card: cardAddress,
        metadata: cardMetadataAddress,
      })
      .rpc();
    await conn.confirmTransaction(tx, "confirmed");

    // check metadata
    const cardMetadata = await fetchDigitalAsset(umi, card);

    expect(cardMetadata.metadata.name).eq(new_name, "Invalid card name");
    expect(cardMetadata.metadata.symbol).eq(new_symbol, "Invalid card symbol");
    expect(cardMetadata.metadata.uri).eq(new_uri, "Invalid URI");
    expect(cardMetadata.mint.publicKey).eq(card, "Invalid card address");
    expect(cardMetadata.metadata.updateAuthority.toString()).eq(
      vault.toString(),
      "Invalid update authority"
    );
    expect(cardMetadata.metadata.isMutable).eq(
      true,
      "Card metadata is immutable"
    );
  });
});
