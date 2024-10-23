///////////////////////////////////////////////////////////
//                       Deploy.rs                       //
//  This file controls deployment to the various systems //
//  that Horizon supports including Kubernetes,          //
//  OpenStack, and Docker                                //
///////////////////////////////////////////////////////////

use docker_api::api as docker_api;
use kube;
use openstack;

enum DeployType {
    docker,         // Horizon comes with many deployment options, openstack and docker are the best for
    swarm,          // compatability on advanced features in the dashboard and autoscalar. Kubernetes and
    kubernetes,     // swarm are provided purely for compatability with pre-existing environments
    openstack,
}

fn deploy (type_: DeployType) {
    match type_ {
        DeployType::docker => {
            println!("Attempting deploy to Docker")

        }

        DeployType::kubernetes => {
            println!("Attempting to deploy to Kubernetes")

        }

        DeployType::openstack => {
            println!("Attempting to deploy on Kubernetes")

        }
    }
}

fn deploy_docker() {
    
}

fn deploy_openstack() {
    
}

fn deploy_kubrenetes() {
    
}