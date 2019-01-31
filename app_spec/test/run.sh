
# 
# Execute the desired tests, retaining tests output and dumping on failure
# 
if [ -z $1 ]; then
    tape test.js regressions.js | tee test.out~ | faucet || ( cat test.out~; false )
else
    tape $1                     | tee test.out~ | faucet || ( cat test.out~; false )
fi 
