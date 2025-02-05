use crate::crypto::{
    compress_public_key, decode_address, ethereum_address_raw, get_recovery_param,
    public_key_to_address,
};
#[cfg(feature = "cosmos")]
use crate::state::CONFIG;
use cosmwasm_std::{Deps, StdError, StdResult};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::str;

#[cfg(feature = "eth")]
pub fn verify_signature(
    deps: Deps,
    _sender: String,
    message: String,
    signature: String,
    signer_address: String,
) -> StdResult<String> {
    let verified = verify_signature_eth(deps, &message, &signature, &signer_address)?;
    if !verified {
        return Err(StdError::generic_err(&format!(
            "signature verification error",
        )));
    }
    Ok(message)
}

#[cfg(feature = "solana")]
pub fn verify_signature(
    deps: Deps,
    _sender: String,
    message: String,
    signature: String,
    signer_address: String,
) -> StdResult<String> {
    let verified = verify_signature_solana(deps, &message, &signature, &signer_address)?;
    if !verified {
        return Err(StdError::generic_err(&format!(
            "signature verification error",
        )));
    }
    Ok(message)
}

#[cfg(feature = "terra")]
pub fn verify_signature(
    _deps: Deps,
    sender: String,
    message: String,
    _signature: String,
    signer_address: String,
) -> StdResult<String> {
    // No signature for terra
    let verified = verify_terra(sender.clone(), signer_address.clone());
    if !verified {
        return Err(StdError::generic_err(&format!(
            "signer address does not match claim. Expected: {} Received: {}",
            sender, signer_address
        )));
    }
    Ok(message)
}

#[cfg(feature = "cosmos")]
pub fn verify_signature(
    deps: Deps,
    _sender: String,
    message: String,
    signature: String,
    signer_address: String,
) -> StdResult<String> {
    let config = CONFIG.load(deps.storage)?;
    let prefix: String = match config.prefix {
        Some(p) => Ok(p),
        None => Err(StdError::generic_err("prefix missing for cosmos airdrop")),
    }?;
    let verified = verify_signature_cosmos(deps, &message, &signature, &signer_address, &prefix)?;
    if !verified {
        return Err(StdError::generic_err(&format!(
            "signature verification error",
        )));
    }
    Ok(message)
}

#[inline]
pub fn verify_signature_eth(
    deps: Deps,
    message: &str,
    signature: &str,
    signer_address: &str,
) -> StdResult<bool> {
    let signer_address = decode_address(signer_address)?;

    // Hashing
    let mut hasher = Keccak256::new();
    hasher.update(format!("\x19Ethereum Signed Message:\n{}", message.len()));
    hasher.update(message);
    let hash = hasher.finalize();

    let signature_u8 = hex::decode(signature)
        .map_err(|_| StdError::generic_err("error decoding hex signature"))?;
    // Decompose signature
    let (v, rs) = match signature_u8.split_last() {
        Some(pair) => pair,
        None => return Err(StdError::generic_err("Signature must not be empty")),
    };
    let recovery = get_recovery_param(*v)?;

    // Verification
    let calculated_pubkey = deps.api.secp256k1_recover_pubkey(&hash, rs, recovery)?;
    let calculated_address = ethereum_address_raw(&calculated_pubkey)?;
    if signer_address != calculated_address {
        return Ok(false);
    }
    let result = deps.api.secp256k1_verify(&hash, rs, &calculated_pubkey);
    match result {
        Ok(verified) => Ok(verified),
        Err(err) => Err(err.into()),
    }
}

#[inline]
pub fn verify_signature_solana(
    deps: Deps,
    message: &str,
    signature: &str,
    signer_address: &str,
) -> StdResult<bool> {
    let signature_u8 =
        &hex::decode(signature).map_err(|_| StdError::generic_err("error decoding signature"))?;
    let public_key = &bs58::decode(signer_address)
        .into_vec()
        .map_err(|_| StdError::generic_err("unable to decode signer address"))?;
    let result = deps
        .api
        .ed25519_verify(message.as_bytes(), signature_u8, public_key);
    match result {
        Ok(verified) => Ok(verified),
        Err(err) => Err(err.into()),
    }
}

pub fn verify_signature_cosmos(
    deps: Deps,
    message: &str,
    signature: &str,
    signer_address: &str,
    prefix: &str,
) -> StdResult<bool> {
    let message_b64 = base64::encode(message);
    let raw_message = format!("{{\"account_number\":\"0\",\"chain_id\":\"\",\"fee\":{{\"amount\":[],\"gas\":\"0\"}},\"memo\":\"\",\"msgs\":[{{\"type\":\"sign/MsgSignData\",\"value\":{{\"data\":\"{}\",\"signer\":\"{}\"}}}}],\"sequence\":\"0\"}}", message_b64, signer_address);
    let mut hasher = Sha256::new();
    hasher.update(raw_message);
    let hash = hasher.finalize();
    let signature_u8 =
        hex::decode(signature).map_err(|_| StdError::generic_err("error decoding signature"))?;
    for i in 0u8..2u8 {
        let recovered_pubkey =
            deps.api
                .secp256k1_recover_pubkey(hash.as_slice(), &signature_u8, i)?;
        let calculated_address = compress_public_key(&recovered_pubkey)?;
        let recovered_address = public_key_to_address(&calculated_address, prefix)?;
        if signer_address == recovered_address {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn verify_terra(sender: String, signer: String) -> bool {
    return sender.eq(&signer);
}
