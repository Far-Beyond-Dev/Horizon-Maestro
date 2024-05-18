const { exec } = require('child_process');
const axios = require('axios');
const toml = require('toml');
const fs = require('fs');

// Read configuration from TOML file
const config = toml.parse(fs.readFileSync('config.toml', 'utf-8'));

// Docker API base URL
const dockerBaseUrl = config.docker.baseUrl;

// Image name and container name
const imageName = config.docker.imageName;
const containerName = config.docker.containerName;

// Local path to Dockerfile and directory containing your Dockerfile and related files
const dockerfilePath = config.dockerfile.path;
const buildContextPath = config.dockerfile.buildContext;

// Build the Docker image
function buildImage() {
  return new Promise((resolve, reject) => {
    const buildCommand = `docker build -t ${imageName} -f ${dockerfilePath} ${buildContextPath}`;

    exec(buildCommand, (error, stdout, stderr) => {
      if (error) {
        reject(error);
      } else if (stderr) {
        reject(new Error(stderr));
      } else {
        console.log(stdout);
        resolve();
      }
    });
  });
}

// Create container endpoint
const createContainerEndpoint = `${dockerBaseUrl}/containers/create`;

// Container configuration
const containerConfig = {
  Image: imageName,
  name: containerName,
  // Additional configuration if needed
};

// Function to create a Docker container
async function createContainer() {
  try {
    // Build the Docker image first
    await buildImage();

    // Make a POST request to create the container
    const response = await axios.post(createContainerEndpoint, containerConfig);
    
    // Check the response status
    if (response.status === 201) {
      console.log('Container created successfully.');
    } else {
      console.error('Failed to create container:', response.statusText);
    }
  } catch (error) {
    console.error('Error creating container:', error.message);
  }
}

// Call the function to create the container
createContainer();
