use futures_util::{StreamExt, SinkExt };
use futures_core::stream::Stream;
use warp::filters::ws::{WebSocket, Message};
use bollard::{ Docker, container::{LogsOptions, LogOutput} };
use std::{
    default::Default, 
    time::{ Duration, SystemTime},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use std::io::{BufWriter, Write, Error};
use std::env::current_dir;
pub mod tests;

/// The query parameters required to execute log streaming.
#[derive(Debug, Serialize, Deserialize)]
pub struct WatchQueryParams {
    /// The container name or id
    pub container_name: String,
    /// The session configured by the client
    pub session_id: String,
    /// How many seconds the websocket server will wait for the client to respond.
    pub timeout: u64,
    /// The key to use in order to access the resources.
    pub stream_key: String,
    #[serde(default)]
    /// Set to true to save logs under /logs directory. Default: false
    pub save_logs: bool,
    #[serde(default)]
    /// This will subract the number of minutes to the current system time. Then it will be used in the since parameter for docker logs.
    pub since_in_minutes: u64,
}

/// Subtract the (N) minutes to the current system time.
/// Use for docker logs "since" parameter.
fn docker_since(mins: u64) -> i64 {
    if mins == 0 {
        return 0;
    }
    
    let mins_to_secs: u64 = mins * 60;
    let d = Duration::from_secs(mins_to_secs);
    match SystemTime::now().checked_sub(d) {
        Some(t) => {
            match t.duration_since(SystemTime::UNIX_EPOCH) {
                Ok(d) => {
                    d.as_secs() as i64
                }
                Err(error) =>{
                    panic!("{:?}",error);
                }
            }
        }
        None => 0
    }
}

/// Establish a connection to docker then execute docker logs
async fn docker_logs(container_name: String, since_in_mins: u64) ->  impl Stream<Item = Result<LogOutput, bollard::errors::Error>> {
    let docker = Docker::connect_with_local_defaults().unwrap();
    let since: i64 = docker_since(since_in_mins);
    docker.logs(
        container_name.as_str(),
        Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            since:since ,
            timestamps: true,
            ..Default::default()
        })
    )
}

/// Send message logs or message to WebSocket Client.
/// This will listen to the container logs via docker logs
pub async fn send_message(socket: WebSocket, container_name: String, since_in_mins: u64, session_id: String, client_stream_key: String, config_stream_key: String, timeout: u64, save_logs: bool) { 
    let (mut tx, mut rx) = socket.split();

    println!("Websocket Connected");
    println!("Session ID: {}",session_id);
    println!("Container Name: {}",container_name);
    println!("Watch Timeout: {}s",timeout);
    if client_stream_key != config_stream_key {
        if let Err(error) = tx.send(Message::text("STREAM_KEY_INVALID")).await {
            eprintln!("STREAM_KEY_INVALID:{:?}",error);
        }
    }
    else {
        let mut logs = docker_logs(container_name.clone(),since_in_mins).await;
        let timeout: u64 = if timeout == 0 {
            30
        } else {
            timeout
        };
        let file_name: String = format!("{}.log",&session_id);
        let mut logger: Logger = Logger::new(&file_name); 
        while let Some(log_result) = logs.next().await { 
            match log_result {
                Ok(log_output) => { 
                    match log_output {
                        LogOutput::Console { message } =>{
                            if save_logs {
                                if let Err(error) = logger.write(&message) {                                                                                                    
                                    let error_message = format!("Unable to write logs to {}.log. Error: {}",&session_id, error);
                                    eprint!("{}",error_message);
                                    if let Err(error) = tx.send(Message::text(error_message)).await {
                                        eprintln!("Unable to send error message to client.Error: {}",error);
                                    }
                                    break

                                }
                            }
                            let message = String::from_utf8_lossy(&message);
                            match tx.send(Message::text(message)).await {
                                Ok(_) => {
                                    if let Err(error) = tokio::time::timeout(Duration::from_secs(timeout), rx.next()).await {                                    
                                        eprintln!("Nothing received from {} with container {}. Error: {:?}",session_id,container_name,error);
                                        break
                                    }
                                }                                
                                Err(error) => {
                                    eprintln!("Unable to send message to {} watching container {}. Error: {:?}",session_id,container_name,error);
                                    break
                                }
                            }
                        }
                        _ => continue
                    };
                },
                Err(error) => {                
                    if let Err(error) = tx.send(Message::text(error.to_string())).await {
                        eprint!("Unable to send message to {}. Error: {}",session_id, error);
                        break
                    }
                }, 
            }
        }
        if let Err(error) = tx.send(Message::text("COMPLETED")).await {
            eprintln!("Unable to send COMPLETED signal to {} for the closing of container {}. Error: {}",session_id,container_name,error)
        }
    }
    match tx.reunite(rx).unwrap().close().await {
        Ok(_) => {
            println!("{} with container {} socket closed.",session_id,container_name);
        }
        Err(error) => {
            eprintln!("Unable to close socket of session {} with container {}. Error: {:?}",session_id,container_name,error);
        }
    };
}

/// Logger struct for handling the writing of logs.
pub struct Logger {
    pub file_handler: BufWriter<fs::File>
}

impl Logger {
    /// This will automatically creates the log directory(if does not exists) then return the log directory path.
    pub fn log_dir() -> String {
        let cdir: String = current_dir().expect("Unable to get current directory for logs.").to_string_lossy().to_string();
        let log_file_path: String = format!("{}/logs/",cdir); 
        if let Err(error) = fs::create_dir_all(&log_file_path) {
            panic!("Unable to create logs base directory. Error: {:?}",error);
        }
        log_file_path
    }

    /// Create a logger instance.
    pub fn new(file_name: &String) -> Self {        
        let log_file_path: String = format!("{}/{}",Logger::log_dir(),&file_name);
        let path: &Path = Path::new(&log_file_path);
        match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => {
                Self {
                    file_handler: BufWriter::new(file)
                }   
            }
            Err(error) => {
                panic!("Unable to create file handler for {}. Error: {:?}",log_file_path,error);
            }
        }
    }

    /// Write the log contents in a file.
    pub fn write(&mut self, data: &[u8] ) -> Result<bool,Error> {
        let f = &mut self.file_handler;
        if let Err(error) = f.write(data) {
            Err(error)
        }
        else{
            match f.flush() {
                Ok(_) => Ok(true),
                Err(error) => Err(error)
            }
        }
    }
}