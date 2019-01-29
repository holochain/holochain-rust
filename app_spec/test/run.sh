if [ -z $1 ]
then
	#faucet test.js regressions.js
	node test.js regressions.js # | faucet
else
	#faucet $1
	node $1 # | faucet
fi
