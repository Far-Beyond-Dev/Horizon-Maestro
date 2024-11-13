//=====================================================
// mod.rs
//-----------------------------------------------------
// This module handles deployment to various systems
// supported by Horizon, including Kubernetes,
// OpenStack, Docker, and Docker Swarm.
//=====================================================

use docker_api::api as docker_api;
use kube as kube_lib;
use openstack as openstack_lib;
use crate::{Config, ContainerConfig, DeploymentConfig, DockerConfig, Host, HostType, MaestroError, NpmConfig};
use rusqlite::{params, Connection};


pub mod docker;
pub mod kube;
pub mod openstack;
pub mod system_api;

enum DeployType {
    Docker,         // Horizon comes with many deployment options, openstack and docker are the best for
    Swarm,          // compatability on advanced features in the dashboard and autoscalar. Kubernetes,
    Kubernetes,     // Openstack, and swarm are provided purely for compatability with pre-existing
    OpenStack,      // environments and are not recommended for new deployments.
}

async fn deploy(type_: DeployType, host: Host, host_type: HostType) {
    let host_clone = host.clone();
    match type_ {
        DeployType::Docker => {
            println!("Attempting deploy to Docker to {}...", host.address);
            let host_type_clone = host_type.clone();
            deploy_docker(host, host_type_clone).await;
        }
        
        DeployType::Kubernetes => {
            println!("Attempting to deploy to Kubernetes");
            deploy_kubrenetes();
        }
        
        DeployType::OpenStack => {
            println!("Attempting to deploy on OpenStack");
            deploy_openstack();
            
        }
        DeployType::Swarm => {
            println!("Attempting to deploy on Docker Swarm");
            deploy_swarm();
        },

    }

    // Add the host to the SQLite database using rusqlite

    let connection = Connection::open("hosts.db").unwrap();
    connection
        .execute(
            "INSERT INTO hosts (address, host_type, status) VALUES (?1, ?2, 'deployed')",
            params![host_clone.address, format!("{:?}", host_type)],
        )
        .unwrap();

    println!("Deployment to node at: {} complete. Added this node to the active nodes database", host_clone.address);
}


async fn deploy_docker(host: Host, host_type: HostType) {
    // SHH into the remote host and run a command to test the connection
    let _ = system_api::run_ssh_command("ls -lah", &host).await;
    // Ensure Docker is installed on the remote host, if not install it
    let _ = docker::ensure_docker_installed_remote(&host).await;
    // Check the host type and deploy the appropriate service
    match host_type {
        HostType::Maestro => {
            let host_clone = host.clone();
            let config = Config {
                docker: DockerConfig {
                    containers: vec![
                        ContainerConfig {
                            image_name: "horizon-dashboard".to_string(),
                            container_name: "horizon-dashboard".to_string(),
                        },
                    ],
                    instances: 1,
                },
                npm: NpmConfig {
                    dashboard_path: "/path/to/dashboard".to_string(),
                },
                deployment: DeploymentConfig {
                    hosts: vec![host],
                    parallel_containers: false,
                },
                cache: None,
            };
            docker::deploy_remotely(&host_clone, &config);
        }
        // Maestro Top Level hosts are not intended to run as child services, therefore
        // we will skip deployment to this host and continue deploying the other services,
        // and print an error message to the console to inform the user.
        HostType::MaestroTopLevel => {
            println!("ERROR: Top Level Maestro hosts are not intended to run as child services.");
            println!("Please use a Maestro or Game Server host type for deployment.");
            println!("We will skip deployment to this host, and contine deploying the other services.");
        },
        HostType::GameServer => todo!(),
    }
}

fn deploy_openstack() {
    
}

fn deploy_kubrenetes() {
    
}

fn deploy_swarm() {
    
}