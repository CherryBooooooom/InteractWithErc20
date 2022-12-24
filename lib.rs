#![cfg_attr(not(feature = "std"), no_std)]


#[ink::contract]
mod test_erc {

    use erc20::{Erc20Ref};
    use ink::env::call::FromAccountId;
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

     // The Meta_Defender result types.
    pub type Result<T> = core::result::Result<T, Error>;


    #[derive(scale::Encode, scale::Decode, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    #[derive(Default)]
    struct CustomStruct {
        value: u128,
    }


    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        NotOfficial,
        TransferError,
    }


    #[ink(storage)]
    pub struct TestErc {
        erc20: Erc20Ref,
        custom: Mapping<AccountId, Vec<CustomStruct>>,
        official: AccountId
    }

    impl TestErc {

        #[ink(constructor)]
        pub fn new(
            erc20: AccountId, 
            official: AccountId, 
        ) -> Self {
            let erc20 = Erc20Ref::from_account_id(erc20);
            let custom = Default::default();
            
            Self {
                erc20,
                custom,
                official, 
            }
        }



        #[ink(message)]
        pub fn insert_value (&mut self, to: AccountId, value:u128, amount:Balance) -> Result<()>{
            let caller = self.env().caller();
            let c = CustomStruct{value};
            match self.custom.get(caller) {
                Some(mut v) => v.push(c),
                None => {
                    self.custom.insert(&caller, &Vec::from([c]));
                }
            }
            match self.erc20.transfer_from(caller, to, amount) {
                Ok(()) => return Ok(()),
                Err(_e) => return Err(Error::TransferError)
            };
        }

        #[ink(message)]
        pub fn change_official(&mut self, to: AccountId ) -> Result<()> {
            let caller = self.env().caller();
            if caller != self.official{
                return Err(Error::NotOfficial)
            }else{
                self.official = to;
                return Ok(())
            }
        }
    }


    
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::TestErcRef;
        use erc20::Erc20Ref;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    
        #[ink_e2e::test(
            additional_contracts = "erc20/Cargo.toml"
        )]
        async fn e2e_delegator(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {

            let erc20_constructor = Erc20Ref::new(
                10_000
            );

            // initiate erc20 by alice
            let erc20_acc_id = client
            .instantiate("erc20", &mut ink_e2e::alice(), erc20_constructor, 0, None)
            .await
            .expect("instantiate failed")
            .account_id;


            // let official being alice, 
            // erc20 was instantiated by its accountid
            let test_constructor = TestErcRef::new(
                erc20_acc_id,
                ink_e2e::alice()
            );

            // initiate testerc20 by alice 
            let test_acc_id = client
                .instantiate("testerc20", &mut ink_e2e::alice(), test_constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            
            // test erc20 approve, 
            // allow test_acc_id to use 10,000 tokens
            let approve = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|contract| contract.approve(test_acc_id,  10_000));

            // message call is Alice
            let call_res = client.call(&mut ink_e2e::alice(), approve, 0, None).await;


            // test erc20 transfer
            // transfer 2,000 to Bob
            let transfer = build_message::<Erc20Ref>(erc20_acc_id.clone())
                .call(|contract| contract.transfer(ink_e2e::bob(),  2_000));

            // Alice transfers 2,000 to Bob
            let transfer_res = client.call(&mut ink_e2e::alice(), transfer, 0, None).await;


            //check Bob's balance after the transfer
            let balance_of = build_message::<Erc20Ref>(erc20_acc_id.clone())
            .call(|contract| contract.balance_of(ink_e2e::bob()));

            let balance_of_result = client
            .call(&mut ink_e2e::charlie(), balance_of, 0, None)
            .await
            .expect("Calling `balance_of` failed");

            let balance_of_bob = balance_of.value.expect("Input is valid, call must not fail.");

            // check whether the balance of bob is equal to 2000 or not
            assert_eq!(balance_of_bob, 2000);



            Ok(())
        }
    
    }


}