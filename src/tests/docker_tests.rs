
#[tokio::test]
async fn test_docker_logs(){
    use crate::docker_logs;
    use crate::Logger;
    use futures_util::StreamExt;
    use bollard::{
        Docker,
        container::{
            RemoveContainerOptions,
            CreateContainerOptions, 
            Config,
            LogsOptions, 
            LogOutput
        },
    };
    let mut logger: Logger = Logger::new("test-docker-logs.log".to_string());
    let docker = Docker::connect_with_local_defaults().unwrap();
    match docker.create_container(Some(CreateContainerOptions{
        name:"test-random-messages".to_string(),
        platform: None
    }),Config {
        hostname: Some("test-random-messages".to_string()),
        tty: Some(false),
        env: Some(vec![            
            "SLEEP_TIME=0.01".to_string(),
            "LOOP_LIMIT=9999".to_string(),
        ]),
        image: Some("random_messages:latest".to_string()),
        ..Default::default()
    }).await {
        Ok(result) => {
            println!("Create Container: {:?}",result);
            match docker.start_container::<String>(&result.id, None ).await {
                Ok(_) => {
                    let container_id: String = result.id.clone();
                    println!("Container ID: {}",container_id);
                    let mut logs = docker_logs(container_id.clone()).await;
                    
                    while let Some(log_result) = logs.next().await {
                        match log_result {
                            Ok(log_output) => {
                                match log_output {
                                    LogOutput::Console { message } =>{
                                        logger.write(&message);
                                    }
                                    _ => continue
                                };
                            },
                            Err(error) => {
                                eprintln!("{:?}",error);
                                assert!(false);
                                break;
                            }
                        }
                    }
                    if let Err(error) = docker.remove_container(container_id.as_str(), Some(RemoveContainerOptions{
                        force:true,
                        ..Default::default()
                    })).await {              
                        eprintln!("docker remove container error: {}",error);
                        assert!(false);
                    }
                    else{
                        assert!(true);
                    }
                }
                Err(error) => {
                    eprintln!("start_container error: {}",error);
                    assert!(false);
                }
            }
        }
        Err(error) => {
            eprintln!("Create Container: {:?}",error);
            assert!(false);
        }
    }
    
}