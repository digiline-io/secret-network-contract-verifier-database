use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::padding::pad_handle_result;
use crate::state::{load, may_load, save, CompilationResult, ADMIN_KEY, BLOCK_SIZE};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    save(
        deps.storage,
        ADMIN_KEY,
        &deps.api.addr_canonicalize(info.sender.as_str())?,
    )?;
    Ok(Response::new().add_attribute("init", "🌈"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let response = match msg {
        ExecuteMsg::SetAdmin { admin } => try_set_admin(deps, info, admin),
        ExecuteMsg::WriteResult {
            code_id,
            repo,
            commit_hash,
            method,
            verified,
        } => try_write_result(deps, info, code_id, repo, commit_hash, method, verified),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_set_admin(deps: DepsMut, info: MessageInfo, admin: Addr) -> StdResult<Response> {
    let current_admin: CanonicalAddr = load(deps.storage, ADMIN_KEY)?;
    let current_admin = deps.api.addr_humanize(&current_admin)?;

    if info.sender != current_admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    save(
        deps.storage,
        ADMIN_KEY,
        &deps.api.addr_canonicalize(admin.as_str())?,
    )?;

    Ok(Response::default())
}

fn try_write_result(
    deps: DepsMut,
    info: MessageInfo,
    code_id: u32,
    repo: String,
    commit_hash: String,
    method: String,
    verified: bool,
) -> StdResult<Response> {
    let admin: CanonicalAddr = load(deps.storage, ADMIN_KEY)?;
    let admin = deps.api.addr_humanize(&admin)?;

    if info.sender != admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let verified_key = format!("codeid_{}", code_id);
    let final_verification_result_key = format!("result_{}", code_id);
    let is_verified = may_load(deps.storage, verified_key.as_bytes())?;

    if let Some(true) = is_verified {
        return Err(StdError::generic_err("Already verified successfully"));
    }

    let specific_key = format!("specific_{} - {}#{}", code_id.clone(), repo, commit_hash,);

    let verified_with_repo = may_load(deps.storage, specific_key.as_bytes())?;
    if let Some(false) = verified_with_repo {
        return Err(StdError::generic_err(
            "Already verified without success with this repo",
        ));
    }

    let result = &CompilationResult {
        code_id,
        repo,
        commit_hash,
        method,
        verified,
    };

    save(deps.storage, specific_key.as_bytes(), &result)?;
    save(deps.storage, verified_key.as_bytes(), &verified)?;

    if verified {
        let mut verified_ids: Vec<u32> =
            may_load(deps.storage, "all_verified".as_bytes())?.unwrap_or_default();

        verified_ids.push(code_id);

        save(
            deps.storage,
            final_verification_result_key.as_bytes(),
            &result,
        )?;
        save(deps.storage, "all_verified".as_bytes(), &verified_ids)?;
    }

    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CheckCodeId { code_id } => to_binary(&query_check_code_id(deps, code_id)?),
        QueryMsg::CheckAllVerified {} => to_binary(&query_check_all_verified(deps)?),
    }
}

fn query_check_code_id(deps: Deps, code_id: u32) -> StdResult<CompilationResult> {
    let verified_key = format!("codeid_{}", code_id);
    let verified_result = may_load(deps.storage, verified_key.as_bytes())?;
    match verified_result {
        Some(true) => {
            let final_verification_result_key = format!("result_{}", code_id);
            let final_verification_result: CompilationResult =
                load(deps.storage, final_verification_result_key.as_bytes())?;

            Ok(final_verification_result)
        }
        _ => Err(StdError::generic_err("Not verified")),
    }
}

fn query_check_all_verified(deps: Deps) -> StdResult<Vec<u32>> {
    let verified_ids: Vec<u32> =
        may_load(deps.storage, "all_verified".as_bytes())?.unwrap_or_default();

    Ok(verified_ids)
}
