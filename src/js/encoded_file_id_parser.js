(function (_0x3f5874, _0x47b412) {
    const _0x45a174 = _0x931e, _0x7089bf = _0x3f5874(); while (!![]) {
        try {
            const _0x355a60 = -parseInt(_0x45a174(0x113)) / 0x1 + parseInt(_0x45a174(0x11f)) / 0x2 + -parseInt(_0x45a174(0x11c)) / 0x3 + parseInt(_0x45a174(0x11a)) / 0x4 + -parseInt(_0x45a174(0x114)) / 0x5 + parseInt(_0x45a174(0x11e)) / 0x6 * (-parseInt(_0x45a174(0x124)) / 0x7) + parseInt(_0x45a174(0x125)) / 0x8 * (parseInt(_0x45a174(0x111)) / 0x9); if (_0x355a60 === _0x47b412) break; else _0x7089bf['push'](_0x7089bf['shift']());
        } catch (_0xb11905) {
            _0x7089bf['push'](_0x7089bf['shift']());
        }
    }
}(_0x79aa, 0x256e7));

function _0x931e(_0x152e3a, _0x39422d) {
    const _0x79aaa3 = _0x79aa();
    return _0x931e = function (_0x931e39, _0x9147d5) {
        _0x931e39 = _0x931e39 - 0x10b;
        let _0x2ffb18 = _0x79aaa3[_0x931e39];
        return _0x2ffb18;
    }, _0x931e(_0x152e3a, _0x39422d);
}

function deobfstr(_0x1dbe96, _0x1ddb3a) {
    const _0x10486f = _0x931e;
    _0x1ddb3a = _0x1ddb3a[_0x10486f(0x121)]();
    let _0x4c518c = '';
    for (let _0x2f1b4f = 0x0; _0x2f1b4f < _0x1dbe96[_0x10486f(0x122)]; _0x2f1b4f += 0x2) {
        const _0xee7ec2 = _0x1dbe96[_0x10486f(0x10b)](_0x2f1b4f, 0x2);
        _0x4c518c += String[_0x10486f(0x110)](parseInt(_0xee7ec2, 0x10) ^ _0x1ddb3a[_0x10486f(0x119)](_0x2f1b4f / 0x2 % _0x1ddb3a[_0x10486f(0x122)]));
    } return _0x4c518c;
}

function _0x79aa() {
    const _0x3dd3d5 = ['fromCharCode', '9aUuACr', 'data', '115516pASduJ', '741435yofCZG', 'body', '#player_iframe', 'html', 'player_iframe', 'charCodeAt', '345872OdgUCN', 'load', '617106FXkKsd', 'background-image:\x20none;', '60126sFIdet', '444032NkiCZl', 'style', 'toString', 'length', 'removeAttr', '49hlVXSB', '3075896gGPSKE', '#hidden', 'substr', '<iframe>', 'height:\x20100%;\x20width:\x20100%;', '#the_frame', 'appendTo']; 
    _0x79aa = function () {
        return _0x3dd3d5;
    }; 
    return _0x79aa();
}
const argv = process.argv;
if (argv.length == 4) {
    let result = deobfstr(argv[3], argv[2]);
    console.log(result);
} else {
    console.log("Expected hash parts arguments");
}