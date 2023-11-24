#!/usr/bin/env sh
set -x
set -e

resim reset

resim set-current-time 2023-11-22T23:01:50Z

XRD=resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3
FAUCET=component_sim1cptxxxxxxxxxfaucetxxxxxxxxx000527798379xxxxxxxxxhkrefh


echo "Admin account"
out=`resim new-account | tee /dev/tty | awk '/Account component address:|Public key:|Private key:|NonFungibleGlobalId:/ {print $NF}'`
echo $out
OWNER_ADDRESS=`echo $out | cut -d " " -f1`
OWNER_PUBKEY=`echo $out | cut -d " " -f2`
OWNER_PVKEY=`echo $out | cut -d " " -f3`
OWNER_NONFUNGIBLEGLOBALID=`resim new-simple-badge --name 'OwnerBadge' | awk '/NonFungibleGlobalId:/ {print $NF}'`

POOL_PACKAGE=`resim publish ../../single_resource_pool/ | tee /dev/tty | awk '/Package:/ {print $NF}'`
echo $POOL_PACKAGE

PRICE_FEED_PACKAGE=`resim publish ../../internal_price_feed | tee /dev/tty | awk '/Package:/ {print $NF}'`
echo $PRICE_FEED_PACKAGE

FAUCET_PACKAGE=`resim publish ../../basic_resource_faucet | tee /dev/tty | awk '/Package:/ {print $NF}'`
echo $FAUCET_PACKAGE

LENDING_MARKET_PACKAGE=`resim publish ../. | tee /dev/tty | awk '/Package:/ {print $NF}'`
echo $LENDING_MARKET_PACKAGE



out=`resim call-function $PRICE_FEED_PACKAGE PriceFeed instantiate | tee /dev/tty | awk '/Component:|Resource:/ {print $NF}'`
PRICE_FEED_COMPONENT_ADDRESS=`echo $out | cut -d " " -f1`
PRICE_FEED_ADMIN_BADGE=`echo $out | cut -d " " -f2`
PRICE_FEED_UPDATER_BADGE=`echo $out | cut -d " " -f4`

out=`resim call-function $FAUCET_PACKAGE Faucet instantiate $PRICE_FEED_COMPONENT_ADDRESS | tee /dev/tty | awk '/Component:|Resource:/ {print $NF}'`
FAUCET_COMPONENT_ADDRESS=`echo $out | cut -d " " -f1`
FAUCET_ADMIN_BADGE=`echo $out | cut -d " " -f2`

echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"lock_fee\" Decimal(\"10\");" > tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"create_proof_of_non_fungibles\"  Address(\"$FAUCET_ADMIN_BADGE\")  Array<NonFungibleLocalId>(NonFungibleLocalId(\"#1#\"));" >> tx.rtm
echo "CALL_METHOD Address(\"$FAUCET_COMPONENT_ADDRESS\") \"create_resource\" \"USDC\" \"USDC\" \"https://res.cloudinary.com/daisvxhyu/image/upload/v1679440531/825_lkjddk.png\"  Decimal(\"1000000\");" >> tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"deposit_batch\" Expression(\"ENTIRE_WORKTOP\");" >> tx.rtm

RESULT=$(resim run "tx.rtm")

export USDC_RESOURCE_ADDRESS=$(echo "$RESULT" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')

echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"lock_fee\" Decimal(\"10\");" > tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"create_proof_of_non_fungibles\"  Address(\"$PRICE_FEED_ADMIN_BADGE\")  Array<NonFungibleLocalId>(NonFungibleLocalId(\"#1#\"));" >> tx.rtm
echo "CALL_METHOD Address(\"$PRICE_FEED_COMPONENT_ADDRESS\") \"admin_update_price\" Address(\"$USDC_RESOURCE_ADDRESS\") Decimal(\"25\");" >> tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"create_proof_of_non_fungibles\"  Address(\"$PRICE_FEED_ADMIN_BADGE\")  Array<NonFungibleLocalId>(NonFungibleLocalId(\"#1#\"));" >> tx.rtm
echo "CALL_METHOD Address(\"$PRICE_FEED_COMPONENT_ADDRESS\") \"admin_update_price\" Address(\"$XRD\") Decimal(\"1\");" >> tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"deposit_batch\" Expression(\"ENTIRE_WORKTOP\");" >> tx.rtm


resim run tx.rtm

# out=`resim call-function $LENDING_MARKET_PACKAGE LendingMarket instantiate Enum<0u8>() Enum<0u8>() | tee /dev/tty | awk '/Component:|Resource:/ {print $NF}'`

echo "CALL_METHOD
    Address(\"$OWNER_ADDRESS\")
    \"lock_fee\"
    Decimal(\"100\");" > tx.rtm

echo "CALL_FUNCTION
    Address(\"$LENDING_MARKET_PACKAGE\")
    \"LendingMarket\"
    \"instantiate\"
    Tuple(10u8,)
;" >> tx.rtm

echo "CALL_METHOD
    Address(\"$OWNER_ADDRESS\")
    \"deposit_batch\"
    Expression(\"ENTIRE_WORKTOP\")
;" >> tx.rtm

RESULT=$(resim run "tx.rtm")


export LENDING_MARKET_COMPONENT_ADDRESS=$(echo "$RESULT" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export LENDING_MARKET_ADMIN_ADDRESS=$(echo "$RESULT" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export LENDING_MARKET_RESERVE_COLLECTOR_BADGE=$(echo "$RESULT" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '2!d')
export LENDING_MARKET_CDP_RESOURCE_ADDRESS=$(echo "$RESULT" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3!d')
export LENDING_BATCH_FLASH_LOAN_RESOURCE_ADDRESS=$(echo "$RESULT" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '4!d')
export LENDING_LIQUIDATION_TERM_RESOURCE_ADDRESS=$(echo "$RESULT" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '4!d')


echo  "LENDING_MARKET_COMPONENT_ADDRESS $LENDING_MARKET_COMPONENT_ADDRESS"
echo  "LENDING_MARKET_ADMIN_ADDRESS $LENDING_MARKET_ADMIN_ADDRESS"
echo  "LENDING_MARKET_RESERVE_COLLECTOR_BADGE $LENDING_MARKET_RESERVE_COLLECTOR_BADGE"
echo  "LENDING_MARKET_CDP_RESOURCE_ADDRESS $LENDING_MARKET_CDP_RESOURCE_ADDRESS"
echo  "LENDING_BATCH_FLASH_LOAN_RESOURCE_ADDRESS $LENDING_BATCH_FLASH_LOAN_RESOURCE_ADDRESS"
echo  "LENDING_LIQUIDATION_TERM_RESOURCE_ADDRESS $LENDING_LIQUIDATION_TERM_RESOURCE_ADDRESS"


echo "CALL_METHOD Address(\"$OWNER_ADDRESS\")\"lock_fee\" Decimal(\"5000\");" > tx.rtm
echo "CALL_METHOD Address(\"$OWNER_ADDRESS\") \"create_proof_of_non_fungibles\" Address(\"$LENDING_MARKET_ADMIN_ADDRESS\") Array<NonFungibleLocalId>(NonFungibleLocalId(\"#1#\"),NonFungibleLocalId(\"#2#\"),NonFungibleLocalId(\"#3#\"),NonFungibleLocalId(\"#4#\"));" >> tx.rtm
echo "CALL_METHOD Address(\"$LENDING_MARKET_COMPONENT_ADDRESS\") \"create_lending_pool\" Address(\"$PRICE_FEED_COMPONENT_ADDRESS\") Address(\"$XRD\") 
    Tuple(
        Decimal(\"0.15\"),
        Decimal(\"0.15\"),
        Decimal(\"0.15\"),
        Decimal(\"0.001\"),
        0u8,
        Decimal(\"0.05\"),
        Decimal(\"1\"),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        5i64,
        15i64,
        240i64
    )
    Tuple(
        Decimal(\"0.05\"),
        Array<Tuple>(
            Tuple(
                Decimal(\"0\"),
                Decimal(\"0.3\")
            ),
            Tuple(
                Decimal(\"0.4\"),
                Decimal(\"3\")
            )
        )
    )
    Tuple(
        Enum<0u8>(),
        Enum<1u8>(
            Decimal(\"0.8\")
        ),
        Map<Address, Decimal>(),
        Map<U8, Decimal>(),
        Decimal(\"0.7\")
    );" >> tx.rtm

echo "CALL_METHOD Address(\"$LENDING_MARKET_COMPONENT_ADDRESS\") \"create_lending_pool\" Address(\"$PRICE_FEED_COMPONENT_ADDRESS\") Address(\"$USDC_RESOURCE_ADDRESS\") 
    Tuple(
        Decimal(\"0.15\"),
        Decimal(\"0.15\"),
        Decimal(\"0.15\"),
        Decimal(\"0.001\"),
        1u8,
        Decimal(\"0.05\"),
        Decimal(\"1\"),
        Enum<0u8>(),
        Enum<0u8>(),
        Enum<0u8>(),
        5i64,
        15i64,
        240i64
    )
    Tuple(
        Decimal(\"0.05\"),
        Array<Tuple>(
            Tuple(
                Decimal(\"0\"),
                Decimal(\"0.5\")
            ),
            Tuple(
                Decimal(\"0.8\"),
                Decimal(\"5\")
            )
        )
    )
    Tuple(
        Enum<0u8>(),
        Enum<1u8>(
            Decimal(\"0.8\")
        ),
        Map<Address, Decimal>(),
        Map<U8, Decimal>(),
        Decimal(\"0\")
    );" >> tx.rtm
echo "CALL_METHOD
    Address(\"$OWNER_ADDRESS\")
    \"deposit_batch\"
    Expression(\"ENTIRE_WORKTOP\");" >> tx.rtm

resim run tx.rtm
