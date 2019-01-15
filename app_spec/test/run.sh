if [ -z $1 ] 
then
	node test.js | faucet
	node regressions.js | faucet
else
	node $1 | faucet
fi
