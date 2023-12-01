
use std::panic::Location;

pub enum LogLevel
{
    Debug,
    Message,
    Warning,
    Error,
}

pub trait Logger
{
    fn log_warning(&self, message : &str);
    fn log_debug(&self, message : &str);
    fn log_message(&self, message : &str);
    fn log_error(&self, message : &str);
}

pub struct ConsoleLogger;

impl ConsoleLogger
{
    fn log(&self, message : &str, level : LogLevel, file : &str, line : u32, column : u32) 
    {
        let level_str = match level 
        {
            LogLevel::Debug => "DEBUG",
            LogLevel::Message => "MESSAGE",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        };
        println!("{} : {}, called from {}: line: {}, column: {}", level_str, message, file, line, column);
    }
}

impl Logger for ConsoleLogger
{
    #[track_caller]
    fn log_warning(&self, message : &str) 
    {
        let loc = Location::caller();
        self.log(message, LogLevel::Warning, loc.file(), loc.line(), loc.column());
    }

    #[track_caller]
    fn log_debug(&self, message : &str) 
    {
        let loc = Location::caller();
        self.log(message, LogLevel::Debug, loc.file(), loc.line(), loc.column());
    }

    #[track_caller]
    fn log_message(&self, message : &str) 
    {
        let loc = Location::caller();
        self.log(message, LogLevel::Message, loc.file(), loc.line(), loc.column());
    }

    #[track_caller]
    fn log_error(&self, message : &str) 
    {
        let loc = Location::caller();
        self.log(message, LogLevel::Error, loc.file(), loc.line(), loc.column());
    }
}