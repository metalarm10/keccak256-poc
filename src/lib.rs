use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use sha3::{Digest, Keccak256};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Compute keccak256 (original Ethereum variant, NOT NIST SHA3-256) over the
    /// given payload. Returns the 32-byte digest.
    #[returns(HashResponse)]
    Hash { payload: Binary },
}

#[cw_serde]
pub struct HashResponse {
    pub digest: Binary,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Hash { payload } => {
            // sha3::Keccak256 is the original (pre-NIST) Keccak used by Ethereum.
            // Do NOT use sha3::Sha3_256 — that's the NIST variant with different
            // padding and produces a different digest for the same input.
            let mut hasher = Keccak256::new();
            hasher.update(payload.as_slice());
            let digest = hasher.finalize();
            to_binary(&HashResponse {
                digest: Binary::from(&digest[..]),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    fn run_hash(payload: &[u8]) -> String {
        let deps = mock_dependencies();
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Hash {
                payload: Binary::from(payload),
            },
        )
        .unwrap();
        let parsed: HashResponse = cosmwasm_std::from_binary(&res).unwrap();
        hex::encode(parsed.digest.as_slice())
    }

    #[test]
    fn keccak_empty_bytes_sanity() {
        // Original Keccak-256 of b"" — the Ethereum reference value.
        // Compare against NIST SHA3-256 which would produce a7ffc6f8...
        assert_eq!(
            run_hash(b""),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }

    #[test]
    fn keccak_hello_world_sanity() {
        // keccak256(b"hello world") — well-known reference value.
        assert_eq!(
            run_hash(b"hello world"),
            "47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad"
        );
    }
}
