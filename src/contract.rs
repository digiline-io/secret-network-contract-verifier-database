use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HandleResult, HumanAddr, InitResponse, Querier,
    QueryResult, StdError, StdResult, Storage,
};
use secret_toolkit::utils::pad_handle_result;

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{load, may_load, save, CompilationResult, ADMIN_KEY, BLOCK_SIZE};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    save(&mut deps.storage, ADMIN_KEY, &env.message.sender)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    let response = match msg {
        HandleMsg::SetAdmin { admin } => try_set_admin(deps, env, admin),
        HandleMsg::WriteResult {
            code_id,
            repo,
            commit_hash,
            method,
            verified,
        } => try_write_result(deps, env, code_id, repo, commit_hash, method, verified),
    };
    pad_handle_result(response, BLOCK_SIZE)
}

fn try_set_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: HumanAddr,
) -> HandleResult {
    let current_admin: HumanAddr = load(&deps.storage, ADMIN_KEY)?;

    if env.message.sender != current_admin {
        return Err(StdError::unauthorized());
    }

    save(&mut deps.storage, ADMIN_KEY, &admin)?;

    Ok(HandleResponse::default())
}

fn try_write_result<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    code_id: u16,
    repo: String,
    commit_hash: String,
    method: String,
    verified: bool,
) -> HandleResult {
    let admin: HumanAddr = load(&deps.storage, ADMIN_KEY)?;

    if env.message.sender != admin {
        return Err(StdError::unauthorized());
    }

    let verified_key = format!("codeid_{}", code_id);
    let final_verification_result_key = format!("result_{}", code_id);
    let is_verified = may_load(&deps.storage, verified_key.as_bytes())?;

    if let Some(true) = is_verified {
        return Err(StdError::generic_err("Already verified successfully"));
    }

    let specific_key = format!("specific_{} - {}#{}", code_id.clone(), repo, commit_hash,);

    let verified_with_repo = may_load(&deps.storage, specific_key.as_bytes())?;
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

    save(&mut deps.storage, specific_key.as_bytes(), &result)?;
    save(&mut deps.storage, verified_key.as_bytes(), &verified)?;

    if verified {
        let mut verified_ids: Vec<u16> =
            may_load(&deps.storage, "all_verified".as_bytes())?.unwrap_or_default();

        verified_ids.push(code_id);

        save(
            &mut deps.storage,
            final_verification_result_key.as_bytes(),
            &result,
        )?;
        save(&mut deps.storage, "all_verified".as_bytes(), &verified_ids)?;
    }

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::CheckCodeId { code_id } => to_binary(&query_check_code_id(deps, code_id)?),
        QueryMsg::CheckAllVerified {} => to_binary(&query_check_all_verified(deps)?),
    }
}

fn query_check_code_id<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    code_id: u16,
) -> StdResult<CompilationResult> {
    let verified_key = format!("codeid_{}", code_id);
    let verified_result = may_load(&deps.storage, verified_key.as_bytes())?;
    match verified_result {
        Some(true) => {
            let final_verification_result_key = format!("result_{}", code_id);
            let final_verification_result: CompilationResult =
                load(&deps.storage, final_verification_result_key.as_bytes())?;

            Ok(final_verification_result)
        }
        _ => Err(StdError::generic_err("Not verified")),
    }
}

fn query_check_all_verified<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<u16>> {
    let verified_ids: Vec<u16> =
        may_load(&deps.storage, "all_verified".as_bytes())?.unwrap_or_default();

    Ok(verified_ids)
}
