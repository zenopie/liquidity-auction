use cosmwasm_std::{
    entry_point, to_binary, from_binary, Binary, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Addr, Uint128, CosmosMsg,
    WasmMsg,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse, Snip20Msg,
    ReceiveMsg, AllocationPercentage, AllocationResponse, AllocationOptionResponse,
};
use crate::state::{STATE, State, DEPOSIT_AMOUNTS, ALLOCATION_OPTIONS, INDIVIDUAL_ALLOCATIONS, 
    Allocation, INDIVIDUAL_PERCENTAGES,
};


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {

    // initiate contract state
    let state = State {
        auction_admin: msg.auction_admin,
        project_snip_contract: msg.project_snip_contract,
        project_snip_hash: msg.project_snip_hash,
        paired_snip_contract: msg.paired_snip_contract,
        paired_snip_hash: msg.paired_snip_hash,
        auction_amount: Uint128::zero(),
        total_deposits: Uint128::zero(),
        auction_active: false,
    };
    STATE.save(deps.storage, &state)?;

    // Register receive for project token
    let project_snip_msg = to_binary(&Snip20Msg::register_receive(env.contract.code_hash))?;
    let project_snip_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.project_snip_contract.into_string(),
        code_hash: state.project_snip_hash,
        msg: project_snip_msg,
        funds: vec![],
    });

    // Register receive for paired liquidity
    let paired_snip_msg = to_binary(&Snip20Msg::register_receive(env.contract.code_hash))?;
    let paired_snip_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.paired_snip_contract.into_string(),
        code_hash: state.paired_snip_hash,
        msg: paired_snip_msg,
        funds: vec![],
    });

    Ok(Response::new()
        .add_messages(vec![project_snip_message, paired_snip_message])
        .add_attribute("action", "instantiate")
        .add_attribute("from", info.sender))
}


#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::EndAuction {} => execute_end_auction(deps, env, info),
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            memo: _,
        } => execute_receive(deps, env, info, sender, from, amount, msg),  
    }
}


pub fn execute_claim(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo
) -> Result<Response, StdError> {

    let mut state = STATE.load(deps.storage)?;
    let claimer = info.sender;

    if state.auction_active {
        return Err(StdError::generic_err("Auction still active"));
    }

    let user_deposit = match DEPOSITS.may_load(deps.storage, &claimer)? {
        Some(deposit) => deposit,
        None => return Err(StdError::generic_err("No deposits found for user")),
    };

    // Calculate user's share
    let user_share = state.auction_amount * user_deposit / state.total_deposits;

    // Remove user's deposit record
    DEPOSITS.remove(deps.storage, &claimer);

    // Transfer project token to user
    let transfer_msg = Snip20Msg::Transfer {
        recipient: claimer.to_string(),
        amount: user_share,
    };
    let msg = to_binary(&transfer_msg)?;
    let transfer_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.project_snip_contract.clone(),
        code_hash: state.project_snip_hash.clone(),
        msg,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(transfer_message)
        .add_attribute("action", "claim")
        .add_attribute("from", info.sender)
        .add_attribute("amount", user_share.to_string()))
}

pub fn execute_end_auction(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo
) -> Result<Response, StdError> {

    // load state
    let mut state = STATE.load(deps.storage)?;

    // check that admin address is valid
    if info.sender != state.auction_admin {
        return Err(StdError::generic_err("invalid admin address"));
    }

    // check that the auction is active
    if !state.auction_active {
        return Err(StdError::generic_err("Auction is not active"));
    }

    // Update the auction status
    state.auction_active = false;
    STATE.save(deps.storage, &state)?;

    // Transfer paired token to admin
    let transfer_msg = Snip20Msg::Transfer {
        recipient: info.sender.to_string(),
        amount: state.total_deposits,
    };
    let msg = to_binary(&transfer_msg)?;
    let transfer_message = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.paired_snip_contract.clone(),
        code_hash: state.paired_snip_hash.clone(),
        msg,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(transfer_message)
        .add_attribute("action", "end_auction")
        .add_attribute("from", info.sender)
        .add_attribute("amount", state.total_deposits))
}


pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, StdError> {

    let msg: ReceiveMsg = from_binary(&msg)?;

    match msg {
        ReceiveMsg::Deposit {} => receive_deposit(deps, env, from, amount),
        ReceiveMsg::BeginAuction {} => receive_begin_auction(deps, env, from, amount),

    }   
}

pub fn receive_deposit(
    deps: DepsMut,
    _env: Env,
    from: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    // check if there is already a deposit under address
    let already_deposited_option: Option<Uint128> = DEPOSIT_AMOUNTS.get(deps.storage, &from);

    // load state
    let mut state = STATE.load(deps.storage)?;

    // check that the snip is the paired snip contract
    let state = STATE.load(deps.storage)?;
    if info.sender != state.paired_snip_contract {
        return Err(StdError::generic_err("invalid snip"));
    }

    //check that the auction is active
    // check that the auction is inactive
    if !state.auction_active {
        return Err(StdError::generic_err("auction is not active"));
    }

    // check if the depositer has already made a deposit
    match already_deposited_option {
        Some(existing_amount) => {
            // Calculate the new total deposit amount
            let new_deposit_amount = existing_amount + amount;

            // Update the deposit amount in storage
            DEPOSIT_AMOUNTS.insert(deps.storage, &from, &new_deposit_amount)?;
        }
        None => {
            // If no existing amount, use the new amount directly
            DEPOSIT_AMOUNTS.insert(deps.storage, &from, &amount)?;
        }
    };

    // Update the total deposits in state
    state.total_deposits += amount;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("from", from)
        .add_attribute("amount", amount.to_string()))
}

pub fn receive_begin_auction(
    deps: DepsMut,
    _env: Env,
    from: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    
    let mut state = STATE.load(deps.storage)?;

    // check for correct snip and admin
    let state = STATE.load(deps.storage)?;
    if info.sender != state.project_snip_contract {
        return Err(StdError::generic_err("invalid snip"));
    }
    if from != state.auction_admin {
        return Err(StdError::generic_err("invalid admin address"));
    }

    // check that the auction is inactive
    if state.auction_active {
        return Err(StdError::generic_err("Auction is active"));
    }

    // Update the total deposits in state
    state.auction_amount += amount;
    state.auction_active = true;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "begin_auction")
        .add_attribute("from", from)
        .add_attribute("amount", amount.to_string()))
}


#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryState {} => to_binary(&query_state(deps)?),
        QueryMsg::QuertyDeposit{address} => to_binary(&query_deposit(deps, address)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse { state: state })
}

pub fn query_deposit(deps: Deps, address: Addr) -> StdResult<AllocationResponse> {

    // Query deposit amount
    let unclaimed_deposit = DEPOSIT_AMOUNTS
        .get(deps.storage, &address)
        .unwrap_or_else(|| Uint128::zero());

    let allocation_response = AllocationResponse {
        unclaimed_deposit: unclaimed_deposit,
    };

    Ok(allocation_response)
}