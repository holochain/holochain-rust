if [ -z $1 ] 
then
	faucet test.js regressions.js
else
	faucet $1
fi
