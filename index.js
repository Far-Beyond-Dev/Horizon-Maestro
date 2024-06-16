const { spawn } = require('child_process');
const axios = require('axios');
const toml = require('toml');
const fs = require('fs');
const path = require('path');

// Read configuration from TOML file
const config = toml.parse(fs.readFileSync('config.toml', 'utf-8'));

// Function to run npm start in Horizon-Dashboard directory
function runNpmStart() {
  return new Promise((resolve, reject) => {
    const child = spawn('npm', ['run', 'start'], {
      shell: true,
      cwd: path.resolve(__dirname, 'Horizon-Dashboard')
    });

    child.stdout.on('data', (data) => {
      console.log(data.toString());
    });

    child.stderr.on('data', (data) => {
      console.error(data.toString());
    });

    child.on('exit', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`NPM start process exited with code ${code}`));
      }
    });

    child.on('error', (err) => {
      reject(err);
    });
  });
}

// Function to create a Docker container
async function createContainer() {
  try {
    // Run npm start in Horizon-Dashboard directory
    await runNpmStart();

    // Make a POST request to create the container (assuming previous configuration remains valid)
    const dockerBaseUrl = config.docker.baseUrl;
    const createContainerEndpoint = `${dockerBaseUrl}/containers/create`;
    const imageName = config.docker.imageName;
    const containerName = config.docker.containerName;
    const containerConfig = {
      Image: imageName,
      name: containerName,
      // Additional configuration if needed
    };

    const response = await axios.post(createContainerEndpoint, containerConfig);
    
    // Check the response status
    if (response.status === 201) {
      console.log('Container created successfully.');
    } else {
      console.error('Failed to create container:', response.statusText);
    }
  } catch (error) {
    console.error('Error:', error.message);
  }
}

// Call the function to create the container
createContainer();
