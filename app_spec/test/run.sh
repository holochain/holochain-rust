if [ -z $1 ] 
then
	node ./test.js | tap-summary
	node ./regressions.js | tap-summary
else
	node $1 | tap-summary
fi
