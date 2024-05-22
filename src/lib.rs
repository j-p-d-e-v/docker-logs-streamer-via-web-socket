use futures_util::{StreamExt, SinkExt };
use futures_core::stream::Stream;
use warp::filters::ws::{WebSocket, Message};
use bollard::{ Docker, container::{LogsOptions, LogOutput} };
use std::{
    time::Duration,
    default::Default,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use std::io::{BufWriter, Write, Error};
pub mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchQueryParams {
    pub container_name: String,
    pub session_id: String,
    pub timeout: u64,
    pub stream_key: String,
    #[serde(default)]
    pub save_logs: bool
}

async fn docker_logs(container_name: String) ->  impl Stream<Item = Result<LogOutput, bollard::errors::Error>> {
    let docker = Docker::connect_with_local_defaults().unwrap();
    docker.logs(
        container_name.as_str(),
        Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            timestamps: true,
            ..Default::default()
        })
    )
}

pub async fn send_message(socket: WebSocket, container_name: String, session_id: String, client_stream_key: String, config_stream_key: String, timeout: u64, save_logs: bool, logs_base_dir: String) { 
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

        let mut logs = docker_logs(container_name.clone()).await;
        let timeout: u64 = if timeout == 0 {
            30
        } else {
            timeout
        };
        let log_file_path: String = format!("{}/{}.log",logs_base_dir,&session_id);
        let mut logger: Logger = Logger::new(log_file_path); 
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



pub struct Logger {
    pub file_handler: BufWriter<fs::File>
}

impl Logger {
    pub fn new(file_path: String) -> Self {
        let path: &Path = Path::new(&file_path);
        match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => {
                Self {
                    file_handler: BufWriter::new(file)
                }   
            }
            Err(error) => {
                panic!("Unable to create file handler for {}. Error: {:?}",file_path,error);
            }
        }
    }
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