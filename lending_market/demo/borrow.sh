source ./create_cdp.sh 

resim set-current-time 2023-11-22T23:01:50Z

echo "CALL_METHOD
    Address(\"$BORROWER_ADDRESS\")
    \"lock_fee\"
    Decimal(\"100\");" > tx.rtm

echo "CALL_METHOD
    Address(\"$BORROWER_ADDRESS\")
    \"create_proof_of_non_fungibles\"
    Address(\"$LENDING_MARKET_CDP_RESOURCE_ADDRESS\")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId(\"#1#\")
    )
;" >> tx.rtm

echo "POP_FROM_AUTH_ZONE
    Proof(\"cdp_proof\")
;" >>  tx.rtm

echo "CALL_METHOD
    Address(\"$LENDING_MARKET_COMPONENT_ADDRESS\")
    \"borrow\"
    Proof(\"cdp_proof\")
    Array<Tuple>(
        Tuple(
            Address(\"$USDC_RESOURCE_ADDRESS\"),
            Decimal(\"200\")
        )
    )
;" >> tx.rtm

echo "CALL_METHOD
    Address(\"$BORROWER_ADDRESS\")
    \"deposit_batch\"
    Expression(\"ENTIRE_WORKTOP\")
;" >> tx.rtm

resim run tx.rtm

resim set-current-time 2023-11-23T23:01:50Z

resim set-default-account $OWNER_ADDRESS  $OWNER_PVKEY $OWNER_NONFUNGIBLEGLOBALID

echo "CALL_METHOD
    Address(\"$OWNER_ADDRESS\")
    \"lock_fee\"
    Decimal(\"100\");" > tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"create_proof_of_non_fungibles\"  Address(\"$PRICE_FEED_ADMIN_BADGE\")  Array<NonFungibleLocalId>(NonFungibleLocalId(\"#1#\"));" >> tx.rtm
echo "CALL_METHOD Address(\"$PRICE_FEED_COMPONENT_ADDRESS\") \"admin_update_price\" Address(\"$XRD\") Decimal(\"0.02\");" >> tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"deposit_batch\" Expression(\"ENTIRE_WORKTOP\");" >> tx.rtm

resim run tx.rtm

