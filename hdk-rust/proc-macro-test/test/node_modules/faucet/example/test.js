var test = require('tape');
function getMessage () {
    var msgs = [ 'yes', 'equals', 'matches', 'yep', 'pretty much', 'woo' ];
    return msgs[Math.floor(Math.random() * msgs.length)];
}

test('beep affirmative', function (t) {
    t.plan(24);
    var i = 0, n = 0;
    var iv = setInterval(function () {
        t.equal(i++, n++, getMessage());
        if (i === 24) clearInterval(iv);
    }, 50);
});

test('boop exterminate', function (t) {
    t.plan(20);
    var i = 0, n = 0;
    var iv = setInterval(function () {
        if ((i + 2) % 8 === 0) {
            t.equal(i, n + 6, getMessage())
        }
        else t.equal(i, n, getMessage());
        i++; n++;
        if (i === 20) clearInterval(iv);
    }, 100);
});
