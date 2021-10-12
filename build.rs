fn main() {
    cc::Build::new().file("r_debug.c").compile("libr_debug.a");
}
