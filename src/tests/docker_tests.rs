/*

Docker Log Testing
1. Prepare a docker image that I can use to stream logs. (limited)
2. Execute docker_logs assert if there are output.
3. If there are output, it should pass.
*/

use futures_util::StreamExt;

#[tokio::test]
async fn test_docker_logs(){
    use crate::docker_logs;
    use bollard::{
        Docker,
        container::{
            RemoveContainerOptions,
            CreateContainerOptions, 
            Config,
        },
    };
    let docker = Docker::connect_with_local_defaults().unwrap();
    match docker.create_container(Some(CreateContainerOptions{
        name:"test-random-messages".to_string(),
        platform: None
    }),Config {
        hostname: Some("test-random-messages".to_string()),
        tty: Some(true),
        env: Some(vec![            
            "SLEEP_TIME=1".to_string(),
            "LOOP_LIMIT=5".to_string(),
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
                    
                    while let Some(log_output) = logs.next().await {
                        match log_output {
                            Ok(_) =>{
                                assert!(true);
                                break
                            }
                            Err(error) => {                    
                                eprintln!("docker logs output error: {}",error);
                                assert!(false);
                                break
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