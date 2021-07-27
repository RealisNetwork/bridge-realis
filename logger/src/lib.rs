pub mod logger {
    use colour::{red, yellow, green, blue};
    use std::fmt::Debug;

    pub enum Colour {
        Red,
        Yellow,
        Green,
        Blue
    }

    fn red(string: String) {
        red!("{:>10}", string);
    }

    fn yellow(string: String) {
        yellow!("{:>10}", string);
    }

    fn green(string: String) {
        green!("{:>10}", string);
    }

    fn blue(string: String) {
        blue!("{:>10}", string)
    }

    // TODO implement
    fn time() {
        print!("[+]");
    }

    pub fn raw_log<T: Debug>(head: String, body: String, tail: &T, colour: Colour) {
        //
        time();
        // Log head part
        match colour {
            Colour::Red =>      red(head),
            Colour::Yellow =>   yellow(head),
            Colour::Green =>    green(head),
            Colour::Blue =>     blue(head)
        }
        // TODO split print
        // Log body part
        println!(" - {:>10} - {:?}", body, tail);
    }

    pub enum Type {
        Error,
        Warning,
        Success,
        Info
    }

    pub fn log<T: Debug>(params: Type, body: String, tail: &T) {
        match params {
            Type::Error =>      raw_log(String::from("Error"), body, tail, Colour::Red),
            Type::Warning =>    raw_log(String::from("Warning"), body, tail, Colour::Yellow),
            Type::Success =>    raw_log(String::from("Success"), body, tail, Colour::Green),
            Type::Info =>       raw_log(String::from("Info"), body, tail, Colour::Blue),
        }
    }
}