
fn caller(func) {
    func();
}

var a = 0;
caller(fn () {
    a = a + 1;
});

print a;