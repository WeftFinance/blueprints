source ./borrow.sh 

out=`resim new-account | tee /dev/tty | awk '/Account component address:|Public key:|Private key:/ {print $NF}'`
LIQUIDATOR_ADDRESS=`echo $out | cut -d " " -f1`
LIQUIDATOR_PUBKEY=`echo $out | cut -d " " -f2`
LIQUIDATOR_PVKEY=`echo $out | cut -d " " -f3`
LIQUIDATOR_NONFUNGIBLEGLOBALID=`resim new-simple-badge --name 'OwnerBadge' | awk '/NonFungibleGlobalId:/ {print $NF}'`

