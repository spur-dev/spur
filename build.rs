fn main() {
    // getting environment variables from .env
    dotenv_build::output(dotenv_build::Config::default()).unwrap();
}
