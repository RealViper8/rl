
fn make_counter() {
    var i = 0;
    fn count() {
        i = i + 1;
        print i;
    }

    return count;
}

var counter1 = make_counter();
var counter2 = make_counter();


counter1();
counter1();

counter2();
counter2();