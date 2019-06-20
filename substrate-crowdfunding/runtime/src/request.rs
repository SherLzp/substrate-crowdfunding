use crate::funding_factory;
use support::{decl_storage, decl_module, StorageValue, StorageMap,
              dispatch::Result, ensure, decl_event, traits::{Currency, ReservableCurrency}};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash, Zero};
use parity_codec::{Encode, Decode};
use rstd::prelude::*;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Request<Hash, Balance, BlockNumber>{
    // the only id of a request
    request_id: Hash,
    // the funding id
    funding_id: Hash,
    // the purpose of the request
    purpose: Vec<u8>,
    // needed money
    cost: Balance,
    // the request deadline
    expiry: BlockNumber,
    // status
    status: u64,
}

pub trait Trait: timestamp::Trait + funding_factory::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

const MAX_REQUESTS_PER_BLOCK: usize = 3;

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance,
        <T as system::Trait>::BlockNumber
    {
        CreateRequest(AccountId, Hash, Hash, Balance, BlockNumber),
        Vote(AccountId, Hash),
        RequestFinalized(Hash, u64, BlockNumber, bool),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as FundingRequest {
        // Global status
        Requests get(requests): map T::Hash => Request<T::Hash, T::Balance, T::BlockNumber>;
        RequestOwner get(owner_of_request): map T::Hash => Option<T::AccountId>;
        // Maximum time limit for the request
        RequestPeriodLimit get(request_period_limit) config(): T::BlockNumber = T::BlockNumber::sa(60480);

        // All requests
        AllRequestArray get(request_by_index): map u64 => T::Hash;
        AllRequestCount get(all_request_count): u64;
        AllRequestIndex: map T::Hash => u64;

        // The funding's requests
        RequestOfFundingArray get(request_of_funding_by_index): map  (T::Hash, u64) => T::Hash;
        RequestOfFundingCount get(request_of_funding_count): map T::Hash => u64;
        RequestOfFundingIndex: map (T::Hash, T::Hash) => u64;

        // The owner's requests
        RequestOfOwnerArray get(request_of_owner): map (T::AccountId, u64) => T::Hash;
        RequestOfOwnerCount get(request_of_owner_count): map T::AccountId => u64;
        RequestOfOwnerIndex: map (T::AccountId, T::Hash) => u64;

        // Requests ending in a block
        RequestsByBlockNumber get(request_expire_at): map T::BlockNumber => Vec<T::Hash>;

        // The amount of money that the project has used
        UsedMoneyOfFunding get(used_money_of_funding): map T::Hash => T::Balance;

        // The number of people who support the request
        SupportedOfRequest get(supported_of_request): map T::Hash => u64;

        // Judge if the user has voted the request
        VotedBefore get(voted_before): map (T::AccountId, T::Hash) => bool;

        // Get the status of a request: 1-success 2-failure
//        RequestStatus get(status_of_request): map T::Hash => u64;

        // Record the number of requests
        Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        fn deposit_event<T>() = default;

        fn create_request(origin, funding_id: T::Hash, purpose: Vec<u8>, cost: T::Balance, expiry: T::BlockNumber) -> Result{
            let sender = ensure_signed(origin)?;

            // Ensure the funding exists
            ensure!(<funding_factory::Module<T>>::is_funding_exists(funding_id), "The funding does not exist");
            // Ensure the funding is success
            ensure!(<funding_factory::Module<T>>::is_funding_success(funding_id) == 1, "The funding does not succeed");
            // Ensure the sender is the owner
            let owner = <funding_factory::Module<T>>::get_funding_owner(funding_id).ok_or("The owner does not exist")?;
            ensure!(sender == owner, "The sender must be the owner of the funding");

            // ensure that the expiry is valid
            ensure!(expiry > <system::Module<T>>::block_number(), "The expiry has to be greater than the current block number");
            ensure!(expiry <= <system::Module<T>>::block_number() + Self::request_period_limit(), "The expiry has be lower than the limit block number");

            let used_balance = Self::used_money_of_funding(&funding_id);
            let total_balance = <funding_factory::Module<T>>::get_funding_total_balance(funding_id);
            let remain_balance = total_balance - used_balance;
            ensure!(remain_balance >= cost, "The remain money is not enough");
            // get the nonce to help generate unique id
            let nonce = <Nonce<T>>::get();

            // generate the unique id
            let request_id = (<system::Module<T>>::random_seed(), &cost, &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            // ensure that the request id is unique
            ensure!(!<Requests<T>>::exists(&funding_id), "Request already exists");

            let new_request = Request{
                request_id,
                funding_id: funding_id.clone(),
                purpose,
                cost,
                expiry,
                status: 0,
            };

            // ensure that the number of requests in the block does not exceed maximum
            let requests = Self::request_expire_at(expiry);
            ensure!(requests.len() < MAX_REQUESTS_PER_BLOCK, "Maximum number of requests is reached for the target block, try another block");

            // Verify adding count is ok first
            // Check adding all request count
            let all_request_count = Self::all_request_count();
            let new_all_request_count = all_request_count.checked_add(1).ok_or("Overflow adding a new request to total requests")?;

            // Check adding requests of funding count
            let request_of_funding_count = Self::request_of_funding_count(funding_id);
            let new_request_of_funding_count = request_of_funding_count.checked_add(1).ok_or("Overflow adding a new request to the funding's requests")?;

            // Check adding requests of owner count
            let request_of_owner_count = Self::request_of_owner_count(&sender);
            let new_request_of_owner_count = request_of_owner_count.checked_add(1).ok_or("Overflow adding a new request to the owner's requests")?;

            // change the global states
            <Requests<T>>::insert(request_id.clone(), new_request.clone());
            <RequestOwner<T>>::insert(request_id.clone(), sender.clone());

            <RequestsByBlockNumber<T>>::mutate(expiry, |requests| requests.push(request_id.clone()));

            // change the state of all requests
            <AllRequestArray<T>>::insert(&all_request_count, request_id.clone());
            <AllRequestCount<T>>::put(new_all_request_count);
            <AllRequestIndex<T>>::insert(request_id.clone(), all_request_count);

            // change the state of funding's requests
            <RequestOfFundingArray<T>>::insert((funding_id.clone(), request_of_funding_count.clone()), request_id.clone());
            <RequestOfFundingCount<T>>::insert(funding_id.clone(), new_request_of_funding_count);
            <RequestOfFundingIndex<T>>::insert((funding_id.clone(), request_id.clone()), request_of_funding_count);

            // change the state of owner's requests
            <RequestOfOwnerArray<T>>::insert((sender.clone(), request_of_owner_count.clone()), request_id.clone());
            <RequestOfOwnerCount<T>>::insert(sender.clone(), new_request_of_owner_count);
            <RequestOfOwnerIndex<T>>::insert((sender.clone(), request_id.clone()), request_of_owner_count);

            // add the nonce
            <Nonce<T>>::mutate(|n| *n += 1);

            // deposit the event
            Self::deposit_event(RawEvent::CreateRequest(sender, funding_id, request_id, cost, expiry));
            Ok(())
        }

        fn support_request(origin, request_id: T::Hash) -> Result{
            let sender = ensure_signed(origin)?;
            // Ensure the request exists
            ensure!(<Requests<T>>::exists(&request_id), "The request does not exist");
            // Get the request
            let mut request = Self::requests(&request_id);
            // Ensure the user is investor
            ensure!(<funding_factory::Module<T>>::is_investor(request.funding_id, sender.clone()), "You are not the investor");
            // Ensure the investor does not vote before
            ensure!(!<VotedBefore<T>>::get((sender.clone(), request_id.clone())), "You have voted before");
            // Ensure the request is not over
            ensure!(request.status == 0, "The request is over");
            // Ensure the request is not expire
            ensure!(<system::Module<T>>::block_number() < request.expiry, "This request is expired.");
            // Get the number of people who have supported the request and add 1
            let supported_request_count = Self::supported_of_request(&request_id);
            let new_supported_request_count = supported_request_count.checked_add(1).ok_or("Overflow adding the number of people who have voted the request")?;
            // Check if the number is bigger than half
            let invested_number = <funding_factory::Module<T>>::get_invested_number(request.funding_id);
            let half_number = invested_number.checked_div(2).ok_or("Error when get half of the invested number")?;
            let supported_count = new_supported_request_count.clone();
            // If the supported_count is bigger than the half, the request is success
            if new_supported_request_count > half_number{
                Self::can_use_balance(request_id, supported_count)?;
            }
            // Change the investor voting status
            <VotedBefore<T>>::insert((sender.clone(), request_id.clone()), true);
            // Change the number of supporters
            <SupportedOfRequest<T>>::insert(request_id.clone(), new_supported_request_count.clone());
            // Deposit the Vote event
            Self::deposit_event(RawEvent::Vote(sender, request_id.clone()));
            Ok(())
        }

        fn on_finalize() {
            // get all the fundings of the block
            let block_number = <system::Module<T>>::block_number();
            let request_hashs = Self::request_expire_at(block_number);

            for request_id in &request_hashs{
                // Get the request
                let mut request = Self::requests(request_id);
                // Check if the request is success before
                if request.status == 1{
                    continue;
                }
                // Else the request fails
                request.status = 2;
                <Requests<T>>::insert(request_id.clone(), request.clone());
                let supported_count = <SupportedOfRequest<T>>::get(request.request_id);
                Self::deposit_event(RawEvent::RequestFinalized(request.request_id, supported_count, request.expiry, false));
            }
        }
    }
}

impl<T:Trait> Module<T>{
    fn can_use_balance(request_id: T::Hash, supported_count: u64) -> Result{
        // Get the request
        let mut request = Self::requests(&request_id);
        let request_balance = request.cost;
        // Ensure that there is enough money
        let used_balance = <UsedMoneyOfFunding<T>>::get(request.funding_id);
        let total_balance = <funding_factory::Module<T>>::get_funding_total_balance(request.funding_id);
        let remain_balance = total_balance - used_balance.clone();
        ensure!(remain_balance >= request_balance, "The remain balance is not enough");
        // Get the owner of the funding
        let owner = <RequestOwner<T>>::get(&request_id).ok_or("No owner of the request")?;
        // Unreserve the request balance
        let _ = <balances::Module<T>>::unreserve(&owner, request_balance.clone());
        // Change the used amount
        let new_used_balance = used_balance + request_balance;
        <UsedMoneyOfFunding<T>>::insert(request.funding_id, new_used_balance);
        // Change the request status
        request.status = 1;
        <Requests<T>>::insert(request_id.clone(), request.clone());
        Self::deposit_event(RawEvent::RequestFinalized(request_id, supported_count, request.expiry, true));
        Ok(())
    }
}

