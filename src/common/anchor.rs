use crate::errors::Error;
use lazy_static::lazy_static;
// TODO: import this from geodata-anchor
use crate::common::msg;
use bip32::XPrv;
use bip39::{Language, Mnemonic, Seed};
use cosmrs::{
  cosmwasm::MsgExecuteContract,
  crypto::secp256k1,
  tx::{self, Msg},
  AccountId, Coin,
};
use cosmwasm_std::Timestamp;

lazy_static! {
  // phrase from juno built-in test account
  static ref SENDER_ACCOUNT_PHRASE: &'static str  = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";
  static ref ACCOUNT_PREFIX: &'static str  = "juno";
  static ref DENOM: &'static str = "ujunox";
  static ref RPC_PORT: u16 = 26657;
  static ref CHAIN_ID: &'static str = "testing";
  static ref MEMO: &'static str = "test memo";
  static ref TIMEOUT_HEIGHT: u16 = 9001u16;
}

fn sender(phrase: &str) -> AccountId {
    // set up private key, public key and account from juno built-in test account
    let mnemonic = Mnemonic::from_phrase(phrase, Language::English).unwrap();
    let seed = Seed::new(&mnemonic, "");
    // let root_privk = XPrv::new(&seed).unwrap();
    let privk = XPrv::derive_from_path(&seed, &"m/44'/118'/0'/0/0".parse().unwrap()).unwrap();
    let bytes = privk.private_key().to_bytes();
    let sender_private_key = secp256k1::SigningKey::from_bytes(bytes.as_slice()).unwrap();
    let sender_public_key = sender_private_key.public_key();
    return sender_public_key.account_id(*ACCOUNT_PREFIX).unwrap();
}

pub async fn anchor_geodata(
  geodata_id: &str,
  account_id: &str,
  hash: &str,
  created_nanos: u64,
) -> Result<(), Error> {
  let created = Timestamp::from_nanos(created_nanos);

  let create_msg = msg::CreateMsg {
    id: geodata_id.to_string(),
    account: account_id.to_string(),
    hash: hash.to_string(),
    created: created,
  };
  let create_msg_json = serde_json::to_string(&create_msg).unwrap();

  let msg: Vec<u8> = serde_json::to_vec(&create_msg_json).unwrap();
  let sender_account_id: AccountId = sender(&SENDER_ACCOUNT_PHRASE);
  //TODO: placeholder for parsing, will supply from geodata_anchor
  let contract: AccountId = sender(&SENDER_ACCOUNT_PHRASE);
  let amount: Coin = Coin {
    amount: 100u8.into(),
    denom: DENOM.parse().unwrap(),
  };
  let msg_execute = MsgExecuteContract {
    sender: sender_account_id,
    contract,
    msg: msg,
    funds: vec![amount.clone()],
  }
  .to_any()
  .unwrap();
  let tx_body = tx::Body::new(vec![msg_execute], MEMO.to_string(), *TIMEOUT_HEIGHT);
  //TODO: send transaction and check
  Ok(())
}

