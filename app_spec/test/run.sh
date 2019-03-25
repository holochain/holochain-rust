if [ -z $1 ]
then
	tape test.js regressions.js | tee test.out~ | faucet || ( cat test.out~; false )
else
	tape $1
fi
