source ./borrow.sh 

resim set-current-time 2023-11-22T23:06:50Z
resim set-default-account $OWNER_ADDRESS  $OWNER_PVKEY $OWNER_NONFUNGIBLEGLOBALID
echo "CALL_METHOD
    Address(\"$OWNER_ADDRESS\")
    \"lock_fee\"
    Decimal(\"100\");" > tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"create_proof_of_non_fungibles\"  Address(\"$PRICE_FEED_ADMIN_BADGE\")  Array<NonFungibleLocalId>(NonFungibleLocalId(\"#1#\"));" >> tx.rtm
echo "CALL_METHOD Address(\"$PRICE_FEED_COMPONENT_ADDRESS\") \"admin_update_price\" Address(\"$USDC_RESOURCE_ADDRESS\") Decimal(\"26\");" >> tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"deposit_batch\" Expression(\"ENTIRE_WORKTOP\");" >> tx.rtm
resim run tx.rtm



out=`resim new-account | tee /dev/tty | awk '/Account component address:|Public key:|Private key:/ {print $NF}'`


LIQUIDATOR_ADDRESS=`echo $out | cut -d " " -f1`
LIQUIDATOR_PUBKEY=`echo $out | cut -d " " -f2`
LIQUIDATOR_PVKEY=`echo $out | cut -d " " -f3`
LIQUIDATOR_NONFUNGIBLEGLOBALID=`resim new-simple-badge --name 'OwnerBadge' | awk '/NonFungibleGlobalId:/ {print $NF}'`

resim set-default-account $LIQUIDATOR_ADDRESS  $LIQUIDATOR_PVKEY $LIQUIDATOR_NONFUNGIBLEGLOBALID

resim set-current-time 2023-11-22T23:50:50Z

echo "CALL_METHOD
    Address(\"$LIQUIDATOR_ADDRESS\")
    \"lock_fee\"
    Decimal(\"100\");" > tx.rtm

echo "CALL_METHOD
    Address(\"$LENDING_MARKET_COMPONENT_ADDRESS\")
    \"start_liquidation\"
    NonFungibleLocalId(\"#1#\")
    Array<Address>(
        Address(\"$XRD\")
    )
    Enum<0u8>();" >> tx.rtm

echo "TAKE_ALL_FROM_WORKTOP
    Address(\"$XRD\")
    Bucket(\"xrd_bucket\");" >> tx.rtm 

echo "CALL_METHOD
    Address(\"$FAUCET_COMPONENT_ADDRESS\")
    \"get_resource\"
    Address(\"$USDC_RESOURCE_ADDRESS\")
    Bucket(\"xrd_bucket\"); " >> tx.rtm 

echo "TAKE_ALL_FROM_WORKTOP
    Address(\"$USDC_RESOURCE_ADDRESS\")
    Bucket(\"udsc_bucket\");" >> tx.rtm 

echo "TAKE_ALL_FROM_WORKTOP
    Address(\"$LENDING_LIQUIDATION_TERM_RESOURCE_ADDRESS\")
    Bucket(\"liquidation_term_bucket\");" >> tx.rtm 

echo "CALL_METHOD
    Address(\"$LENDING_MARKET_COMPONENT_ADDRESS\")
    \"end_liquidation\"
    Array<Bucket>(
        Bucket(\"udsc_bucket\")
    )
    Bucket(\"liquidation_term_bucket\");" >> tx.rtm

echo "CALL_METHOD
    Address(\"$LIQUIDATOR_ADDRESS\")
    \"deposit_batch\"
    Expression(\"ENTIRE_WORKTOP\");"  >> tx.rtm

resim run tx.rtm