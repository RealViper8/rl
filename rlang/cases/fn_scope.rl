var a = "global";

{
    fn showA() {
        print a;
    }

    showA();
    var a = "block";
    showA();
}