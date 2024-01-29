var exist = function (x) {
    return x != null && typeof (x) != 'undefined' && x != 'undefined'
};

var v = {
    bk0: "%?6497.[:4",
    bk1: "=(=:19705/",
    bk2: ":]&*1@@1=&",
    bk3: "33-*.4/9[6",
    bk4: "*,4).(_)()",
    file3_separator: "/@#@/"
}
function decode(x) {
    var a;
    a = x.substr(2);
    for (var i = 4; i > -1; i--) {
        if (exist(v["bk" + i])) {
            if (v["bk" + i] != "") {
                a = a.replace(v.file3_separator + b1(v["bk" + i]), "");
            }
        }
    } try {
        a = b2(a);
    } catch (e) {
        console.log("Err: ", e);
        a = "";
    }
    function b1(str) {
        return Buffer.from(encodeURIComponent(str).replace(/%([0-9A-F]{2})/g,
            function toSolidBytes(match, p1) {
                return String.fromCharCode("0x" + p1);
            })).toString('base64');
    }
    function b2(str) {
        let x = Buffer.from(str, 'base64').toString('binary').split("").map(function (c) {
            return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
        }).join("");
        return decodeURIComponent(x);
    }
    return a;
}

const argv = process.argv;
if (argv.length == 3) {
    var decoded_url = decode(argv[2]);
    console.log(decoded_url);
} else {
    console.log("Expected encoded file id argument");
}