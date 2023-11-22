use transaction::builder::ManifestBuilder;
use crate::helpers::{ methods::*,  init::TestHelper};
use radix_engine_interface::prelude::*;


#[test]
pub fn test_flash_loan(){
    let mut helper = TestHelper::new();
     
    // SET UP A LP PROVIDER
     let (lp_user_key, _, lp_user_account) = helper.test_runner.new_allocated_account();
     helper.test_runner.load_account_from_faucet(lp_user_account);
     helper.test_runner.load_account_from_faucet(lp_user_account);
     
     // Provide 25000 XRD
     market_contribute(&mut helper, lp_user_key, lp_user_account, XRD, dec!(25_000))
     .expect_commit_success();

    
    // FLASH LOAN 
    let (user_public_key, _, user_account_address) = helper.test_runner.new_allocated_account();
    helper.test_runner.load_account_from_faucet(user_account_address);
    
    let mut loan_amounts :IndexMap<ResourceAddress, Decimal> = IndexMap::new(); 
    let loan_amount = Decimal::from(1000);
    loan_amounts.insert(XRD , loan_amount); 
    
    let manifest_builder = ManifestBuilder::new();  
    
   
    let manifest = manifest_builder 
    .lock_fee_from_faucet()

    // TAKE FLASH LOAN
    .call_method(
        helper.market.market_component_address,
        "take_batch_flashloan",
        manifest_args!(loan_amounts),
    ).take_from_worktop(XRD, loan_amount, "xrd_bucket")
    .with_name_lookup(|builder, lookup| {
        let xrd_buket = lookup.bucket("xrd_bucket");
        builder.call_method(
            helper.faucet.faucet_component_address,
            "get_resource",
            manifest_args!(helper.faucet.usdc_resource_address, xrd_buket),
        )
    }) .take_from_worktop(
        helper.market.batch_flashloan_resource_address,
        Decimal::from(1),
        "flash_loan_term_bucket"
    )
    .with_name_lookup(|builder, _lookup| {
        let flash_loan_term_bucket = _lookup.bucket("flash_loan_term_bucket");
        let mut payments : Vec<(ResourceAddress, Decimal)> = Vec::new();
        payments.push((XRD,loan_amount + Decimal::from(100))); 
        let mut payment_buckets = Vec::<ManifestBucket>::new();    
        let (_, newbuilder) =
            payments
                .iter()
                .fold((0, builder), |(i, builder), (res_address, amount)| {
                    (
                        i,
                        builder
                            .withdraw_from_account(user_account_address, *res_address, *amount)
                            .take_all_from_worktop(
                                *res_address,
                                format!("payment_bucket_{}", i),
                            )
                            .with_name_lookup(|builder, lookup| {
                                payment_buckets
                                    .push(lookup.bucket(format!("payment_bucket_{}", i)));
                                builder
                            }),
                    )
                });

        newbuilder.call_method(
            helper.market.market_component_address,
            "repay_batch_flashloan",
            manifest_args!(payment_buckets, flash_loan_term_bucket),
        ).deposit_batch(user_account_address)
    }).build();

    // market_take_batch_flashloan(&mut helper,user_public_key, user_account_address, loan_amounts,  &mut manifest_builder) ; 
    // get_resource_flash_loan(&mut helper, user_public_key, user_account_address, loan_amount,&mut  manifest_builder); 
    //market_repay_batch_flashloan(&mut helper, user_public_key, user_account_address, payments, &mut manifest_builder);
    
    helper.test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&user_public_key)],
    ).expect_commit_success(); 

}
