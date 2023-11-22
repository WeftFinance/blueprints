source ./baseline.sh 

out=`resim new-account | tee /dev/tty | awk '/Account component address:|Public key:|Private key:/ {print $NF}'`
LP_PROVIDER_ADDRESS=`echo $out | cut -d " " -f1`
LP_PROVIDER_PUBKEY=`echo $out | cut -d " " -f2`
LP_PROVIDER_PVKEY=`echo $out | cut -d " " -f3`
LP_PROVIDER_NONFUNGIBLEGLOBALID=`resim new-simple-badge --name 'OwnerBadge' | awk '/NonFungibleGlobalId:/ {print $NF}'`

resim set-default-account $LP_PROVIDER_ADDRESS  $LP_PROVIDER_PVKEY $LP_PROVIDER_NONFUNGIBLEGLOBALID

out=`resim call-method $FAUCET free`
echo $out
out=`resim call-method $FAUCET free`
echo $out
out=`resim call-method $FAUCET free`
echo $out

echo "CALL_METHOD Address(\"$LP_PROVIDER_ADDRESS\") \"lock_fee\" Decimal(\"100\");" > tx.rtm 

echo "CALL_METHOD
    Address(\"$LP_PROVIDER_ADDRESS\")
    \"withdraw\"
    Address(\"$XRD\")
    Decimal(\"25000\");" >> tx.rtm 

echo "TAKE_ALL_FROM_WORKTOP
    Address(\"$XRD\")
    Bucket(\"xrd_bucket\");" >> tx.rtm 

echo "CALL_METHOD
    Address(\"$FAUCET_COMPONENT_ADDRESS\")
    \"get_resource\"
    Address(\"$USDC_RESOURCE_ADDRESS\")
    Bucket(\"xrd_bucket\"); " >> tx.rtm 

echo "TAKE_ALL_FROM_WORKTOP Address(\"$USDC_RESOURCE_ADDRESS\") Bucket(\"usd_bucket\");" >> tx.rtm
echo "CALL_METHOD Address(\"$LENDING_MARKET_COMPONENT_ADDRESS\") \"contribute\"  Bucket(\"usd_bucket\");" >> tx.rtm
echo "CALL_METHOD Address(\"$LP_PROVIDER_ADDRESS\") \"deposit_batch\" Expression(\"ENTIRE_WORKTOP\");" >> tx.rtm


resim run tx.rtm