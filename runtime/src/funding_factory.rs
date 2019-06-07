use support::{decl_storage, decl_module, StorageValue, StorageMap,
              dispatch::Result, ensure, decl_event, traits::{Currency, ReservableCurrency}};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash, Zero};
use parity_codec::{Encode, Decode};
use rstd::prelude::*;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Funding<Hash, AccountId, Balance, BlockNumber>{
    // the only id of a funding
    funding_id: Hash,
    // funding raiser
    manager: AccountId,
    // funding project's name
    project_name: Vec<u8>,
    // target money to raise
    target_money: Balance,
    // the funding project deadline
    expiry: BlockNumber,
}

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

const MAX_FUNDINGS_PER_BLOCK: usize = 3;

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance,
        <T as system::Trait>::BlockNumber
    {
        CreateFunding(AccountId, Hash, Balance, Balance, BlockNumber),
        Invest(Hash, AccountId, Balance),
        FundingFinalized(Hash, Balance, BlockNumber, bool),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as FundingFactory {
        // Global state
        Fundings get(funding_by_id): map T::Hash => Funding<T::Hash, T::AccountId, T::Balance, T::BlockNumber>;
        // Owner of a funding project
        FundingOwner get(owner_of): map T::Hash => Option<T::AccountId>;
        // Maximum time limit for the project
        FundingPeriodLimit get(funding_period_limit) config(): T::BlockNumber = T::BlockNumber::sa(777600);

        // Projects ending in a block
        FundingsByBlockNumber get(funding_expire_at): map T::BlockNumber => Vec<Funding<T::Hash, T::AccountId, T::Balance, T::BlockNumber>>;

        // All funding state
        AllFundingArray get(funding_by_index): map u64 => T::Hash;
        AllFundingCount get(all_funding_count): u64;
        AllFundingIndex: map T::Hash => u64;

        // The user's funding state
        OwnedFundingArray get(funding_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedFundingCount get(owned_funding_count): map T::AccountId => u64;
        OwnedFundingIndex: map (T::AccountId, T::Hash) => u64;

        // The investor's funding state
        InvestedFundings get(invested_funding): map T::AccountId => Vec<Funding<T::Hash, T::AccountId, T::Balance, T::BlockNumber>>;
        InvestedFundingsArray get(invested_funding_by_index): map (T::AccountId, u64) => T::Hash;
        InvestedFundingsCount get(invested_funding_count): map T::AccountId => u64;
        InvestedFundingsIndex: map (T::AccountId, T::Hash) => u64;

        // The investor invested how much money for a project
        InvestAmount get(invest_amount_of): map (T::Hash, T::AccountId) => T::Balance;
        // Investors that who had invested the project before
        InvestAccounts get(invest_accounts): map T::Hash => Vec<T::AccountId>;

        // The total amount of money the project has got
        FundingSupportedAmount get(total_amount_of_funding): map T::Hash => T::Balance;

        // Get the status of a funding project: true-success false-fail
        FundingStatus get(funding_status): map T::Hash => bool;

        // Record the number of funding
        Nonce: u64;
    }
}

decl_module!{
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        fn deposit_event<T>() = default;

        /// Create a funding
        fn create_funding(origin, project_name: Vec<u8>, target_money: T::Balance, support_money: T::Balance, expiry: T::BlockNumber) -> Result {
            // get the sender
            let sender = ensure_signed(origin)?;
            // get the nonce to help generate unique id
            let nonce = <Nonce<T>>::get();
            // generate the unique id
            let funding_id = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);
            // ensure that the funding id is unique
            ensure!(!<FundingOwner<T>>::exists(&funding_id), "Funding already exists");
            // ensure that the support_money less than target_money
            ensure!(support_money <= target_money, "You already have enough money");
            // create a new funding
            let new_funding = Funding{
                funding_id: funding_id.clone(),
                manager: sender.clone(),
                project_name: project_name,
                target_money: target_money,
                expiry: expiry,
            };
            // ensure that the expiry is valid
            ensure!(expiry > <system::Module<T>>::block_number(), "The expiry has to be greater than the current block number");
            ensure!(expiry <= <system::Module<T>>::block_number() + Self::funding_period_limit(), "The expiry has be lower than the limit block number");

            // ensure that the number of fundings in the block does not exceed maximum
            let fundings = Self::funding_expire_at(expiry);
            ensure!(fundings.len() < MAX_FUNDINGS_PER_BLOCK, "Maximum number of fundings is reached for the target block, try another block");

            // change the global states
            <Fundings<T>>::insert(funding_id.clone(), new_funding.clone());
            <FundingOwner<T>>::insert(funding_id.clone(), sender.clone());

            <FundingsByBlockNumber<T>>::mutate(expiry, |fundings| fundings.push(new_funding.clone()));

            let all_funding_count = Self::all_funding_count();
            let new_all_funding_count = all_funding_count.checked_add(1).ok_or("Overflow adding a new funding to total fundings")?;

            // change the state of all fundings
            <AllFundingArray<T>>::insert(&all_funding_count, funding_id.clone());
            <AllFundingCount<T>>::put(new_all_funding_count);
            <AllFundingIndex<T>>::insert(funding_id.clone(), all_funding_count);

            let owned_funding_count = Self::owned_funding_count(&sender);
            let new_owned_funding_count = owned_funding_count.checked_add(1).ok_or("Overflow adding a new funding to account balance")?;

            // change the state of owner related fundings
            <OwnedFundingArray<T>>::insert((sender.clone(), owned_funding_count.clone()), funding_id.clone());
            <OwnedFundingCount<T>>::insert(&sender, new_owned_funding_count);
            <OwnedFundingIndex<T>>::insert((sender.clone(), funding_id.clone()), owned_funding_count);

            if support_money > T::Balance::sa(0) {
                Self::not_invest_before(sender.clone(), funding_id.clone(), support_money.clone())?;
            }
            // add the nonce
            <Nonce<T>>::mutate(|n| *n += 1);

            // deposit the event
            Self::deposit_event(RawEvent::CreateFunding(sender, funding_id, target_money, support_money, expiry));
            Ok(())
        }

        fn invest(origin, funding_id: T::Hash, invest_amount: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(funding_id).ok_or("No owner of the funding")?;
            ensure!(owner != sender, "You can't invest for your own project");

            // The investor had not invested the project before
            if !<InvestAmount<T>>::exists((funding_id.clone(), sender.clone())){
                Self::not_invest_before(sender.clone(), funding_id.clone(), invest_amount.clone());
            }else{
                Self::invest_before(sender.clone(), funding_id.clone(), invest_amount.clone());
            }

            // deposit the event
            Self::deposit_event(RawEvent::Invest(funding_id, sender, invest_amount));

            Ok(())
        }

        fn on_finalize() {
            // get all the fundings of the block
            let block_number = <system::Module<T>>::block_number();
            let fundings = Self::funding_expire_at(block_number);


            'outer: for funding in &fundings{
                // Get the amount of money that the funding had raised
                let amount_of_funding = Self::total_amount_of_funding(funding.funding_id);
                // If the money had raised more than the target_money, then the funding is success
                if amount_of_funding >= funding.target_money{
                    // Make the status success
                    <FundingStatus<T>>::insert(funding.funding_id.clone(), true);
                    // Get the owner of the funding
                    let _owner = Self::owner_of(funding.funding_id);
                    match _owner {
                        Some(owner) => {
                            // Get all the investors
                            let investors = Self::invest_accounts(funding.funding_id);
                            let mut no_error = true;
                            // Iterate every investor, unreserve the money that he/she had invested and transfer it to owner
                            'inner: for investor in &investors{
                                let invest_balance = Self::invest_amount_of((funding.funding_id, investor.clone()));
                                let _ = <balances::Module<T>>::unreserve(&investor, invest_balance.clone());
                                // If the investor is owner, just unreserve the money
                                if investor == &owner{ continue;}
                                let _currency_transfer = <balances::Module<T> as Currency<_>>::transfer(&investor, &owner, invest_balance);
                                match _currency_transfer {
                                    Err(_e) => {
                                        no_error = false;
                                        break 'inner;
                                    },
                                    Ok(_v) => {}
                                }
                            }
                            // If all the processes are right then reserve all money of the funding
                            if no_error {
                                let _ = <balances::Module<T>>::reserve(&owner, amount_of_funding);
                                // deposit the event
                                Self::deposit_event(RawEvent::FundingFinalized(funding.funding_id, amount_of_funding, block_number, true));
                            }
                        },
                        None => continue,
                    }
                }else{ // refund all of the money
                    // Make the status fail
                    <FundingStatus<T>>::insert(funding.funding_id.clone(), false);
                    let funding_accounts = Self::invest_accounts(funding.funding_id);
                    // refund all the money
                    for account in funding_accounts {
                        let invest_balance = Self::invest_amount_of((funding.funding_id, account.clone()));
                        let _ = <balances::Module<T>>::unreserve(&account, invest_balance);
                    }
                    // deposit the event
                    Self::deposit_event(RawEvent::FundingFinalized(funding.funding_id, amount_of_funding, block_number, false));
                }
            }
        }
    }
}

impl<T: Trait> Module<T> {

    //The investor had invested the project before
    fn invest_before(sender: T::AccountId, funding_id: T::Hash, invest_amount: T::Balance) -> Result{
        // ensure the funding exists
        ensure!(<Fundings<T>>::exists(funding_id), "The funding does not exist");
        // ensure the investor has enough money
        ensure!(<balances::Module<T>>::free_balance(sender.clone()) >= invest_amount, "You don't have enough free balance for investing for the funding");

        // get the funding
        let funding = Self::funding_by_id(&funding_id);
        // ensure that the project is valid to invest
        ensure!(<system::Module<T>>::block_number() < funding.expiry, "This funding is expired.");

        // reserve the amount of money
        <balances::Module<T>>::reserve(&sender, invest_amount)?;

        let amount_of_investor_on_funding = Self::invest_amount_of((funding_id.clone(), sender.clone()));
        let new_amount_of_investor_on_funding = amount_of_investor_on_funding + invest_amount.clone();

        //change the amount of the investor has invested
        <InvestAmount<T>>::insert((funding_id, sender), new_amount_of_investor_on_funding.clone());

        // get the total amount of the project and add invest_amount
        let amount_of_funding = Self::total_amount_of_funding(&funding_id);
        let new_amount_of_funding = amount_of_funding + invest_amount;

        // change the total amount of the project has collected
        <FundingSupportedAmount<T>>::insert(&funding_id, new_amount_of_funding);

        Ok(())
    }

    // The investor doesn't invest the project before
    fn not_invest_before(sender: T::AccountId, funding_id: T::Hash, invest_amount: T::Balance) -> Result{
        // ensure the funding exists
        ensure!(<Fundings<T>>::exists(funding_id), "The funding does not exist");
        // ensure that the investor has enough money
        ensure!(<balances::Module<T>>::free_balance(sender.clone()) >= invest_amount, "You don't have enough free balance for investing for the funding");

        // get the number of projects that the investor had invested and add it
        let invested_funding_count = Self::invested_funding_count(&sender);
        let new_invested_funding_count = invested_funding_count.checked_add(1).ok_or("Overflow adding a new invested funding")?;

        // get the funding
        let funding = Self::funding_by_id(&funding_id);
        // ensure that the project is valid to invest
        ensure!(<system::Module<T>>::block_number() < funding.expiry, "This funding is expired.");

        // reserve the amount of money
        <balances::Module<T>>::reserve(&sender, invest_amount)?;

        //change the state of invest related fields
        <InvestAmount<T>>::insert((funding_id.clone(), sender.clone()), invest_amount.clone());
        <InvestAccounts<T>>::mutate(&funding_id, |accounts| accounts.push(sender.clone()));

        // change the state of invest related fields
        <InvestedFundings<T>>::mutate(&sender,|total_fundings| total_fundings.push(funding.clone()));
        <InvestedFundingsArray<T>>::insert((sender.clone(), invested_funding_count), funding_id.clone());
        <InvestedFundingsCount<T>>::insert(&sender, new_invested_funding_count);
        <InvestedFundingsIndex<T>>::insert((sender.clone(), funding_id.clone()), invested_funding_count);

        // get the total amount of the project and add invest_amount
        let amount_of_funding = Self::total_amount_of_funding(&funding_id);
        let new_amount_of_funding = amount_of_funding + invest_amount;

        // change the total amount of the project has collected
        <FundingSupportedAmount<T>>::insert(&funding_id, new_amount_of_funding);

        Ok(())
    }
}