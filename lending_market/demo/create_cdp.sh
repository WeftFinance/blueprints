source ./contribute.sh

out=`resim new-account | tee /dev/tty | awk '/Account component address:|Public key:|Private key:/ {print $NF}'`
BORROWER_ADDRESS=`echo $out | cut -d " " -f1`
BORROWER_PUBKEY=`echo $out | cut -d " " -f2`
BORROWER_PVKEY=`echo $out | cut -d " " -f3`
BORROWER_NONFUNGIBLEGLOBALID=`resim new-simple-badge --name 'OwnerBadge' | awk '/NonFungibleGlobalId:/ {print $NF}'`

resim set-default-account $BORROWER_ADDRESS  $BORROWER_PVKEY $BORROWER_NONFUNGIBLEGLOBALID

echo "CALL_METHOD
    Address(\"$BORROWER_ADDRESS\")
    \"lock_fee\"
    Decimal(\"100\");" > tx.rtm

echo "CALL_METHOD
    Address(\"$BORROWER_ADDRESS\")
    \"withdraw\"
    Address(\"$XRD\")
    Decimal(\"9500\");"  >> tx.rtm 

echo "TAKE_ALL_FROM_WORKTOP
    Address(\"$XRD\")
    Bucket(\"res_bucket_0\")
;" >> tx.rtm 

echo "CALL_METHOD
    Address(\"$LENDING_MARKET_COMPONENT_ADDRESS\")
    \"create_cdp\"
    Enum<0u8>()
    Enum<0u8>()
    Enum<0u8>()
    Array<Bucket>(
        Bucket(\"res_bucket_0\")
    )
;" >> tx.rtm


echo "CALL_METHOD
    Address(\"$BORROWER_ADDRESS\")
    \"deposit_batch\"
    Expression(\"ENTIRE_WORKTOP\")
;" >> tx.rtm

resim run tx.rtm