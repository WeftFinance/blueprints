#!/usr/bin/env sh
set -x
set -e

resim reset

XRD=resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3
FAUCET=component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh


echo "Admin account"
out=`resim new-account | tee /dev/tty | awk '/Account component address:|Public key:|Private key:|NonFungibleGlobalId:/ {print $NF}'`
echo $out
OWNER_ADDRESS=`echo $out | cut -d " " -f1`
OWNER_PUBKEY=`echo $out | cut -d " " -f2`
OWNER_PVKEY=`echo $out | cut -d " " -f3`
OWNER_NONFUNGIBLEGLOBALID=`resim new-simple-badge --name 'OwnerBadge' | awk '/NonFungibleGlobalId:/ {print $NF}'`

LSU_POOL_PACKAGE_ADDRESS=`resim publish ../../caviar_lsu_pool_component_faucet/ | tee /dev/tty | awk '/Package:/ {print $NF}'`
echo $LSU_POOL_PACKAGE_ADDRESS

CAVIAR_LSU_PRICE_FEED_PROXY_PACKAGE=`resim publish ../../caviar_lsu_price_feed_proxy | tee /dev/tty | awk '/Package:/ {print $NF}'`
echo $CAVIAR_LSU_PRICE_FEED_PROXY_PACKAGE

LIQUIDITY_TOKEN_TOTAL_SUPPLY=10000000 
DEX_VALUATION_XRD=10000001

out=`resim call-function $LSU_POOL_PACKAGE_ADDRESS CaviarLsuPoolComponentFaucet instantiate $LIQUIDITY_TOKEN_TOTAL_SUPPLY  $DEX_VALUATION_XRD | tee /dev/tty | awk '/Component:|Resource:/ {print $NF}'`
LSU_POOL_COMPONENT_ADDRESS=`echo $out | cut -d " " -f1`

SUPPLY=100000000

out=`resim new-token-fixed $SUPPLY | tee /dev/tty | awk '/Component:|Resource:/ {print $NF}'`
RESOURCE_ADDRESS=`echo $out | cut -d " " -f1`

out=`resim call-function $CAVIAR_LSU_PRICE_FEED_PROXY_PACKAGE CaviarLsuPriceFeedProxy instantiate $RESOURCE_ADDRESS $LSU_POOL_COMPONENT_ADDRESS | tee /dev/tty | awk '/Component:|Resource:/ {print $NF}'`
CAVIAR_LSU_PRICE_FEED_PROXY_COMPONENT_ADDRESS=`echo $out | cut -d " " -f1`

echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"lock_fee\" Decimal(\"10\");" > tx.rtm
echo "CALL_METHOD Address(\"$CAVIAR_LSU_PRICE_FEED_PROXY_COMPONENT_ADDRESS\") \"get_price\" Address(\"$RESOURCE_ADDRESS\");" >> tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"deposit_batch\" Expression(\"ENTIRE_WORKTOP\");" >> tx.rtm

RESULT=$(resim run "tx.rtm")
