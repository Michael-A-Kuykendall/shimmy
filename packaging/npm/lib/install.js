const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const https = require('https');

const GITHUB_REPO = 'Michael-A-Kuykendall/shimmy';
const BINARY_NAME = process.platform === 'win32' ? 'shimmy.exe' : 'shimmy';

function getPlatformInfo() {
  const platform = process.platform;
  const arch = process.arch;
  
  const platformMap = {
    'win32': 'windows',
    'darwin': 'darwin', 
    'linux': 'linux'
  };
  
  const archMap = {
    'x64': 'amd64',
    'arm64': 'arm64'
  };
  
  return {
    platform: platformMap[platform],
    arch: archMap[arch],
    extension: platform === 'win32' ? '.exe' : ''
  };
}

async function downloadBinary() {
  console.log('🔄 Installing Shimmy binary...');
  
  const { platform, arch, extension } = getPlatformInfo();
  
  if (!platform || !arch) {
    throw new Error(`Unsupported platform: ${process.platform}-${process.arch}`);
  }
  
  const packageJson = require('../package.json');
  const version = packageJson.version;
  
  // Construct download URL
  const filename = `shimmy-${platform}-${arch}${extension}`;
  const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/v${version}/${filename}`;
  
  // Create bin directory
  const binDir = path.join(__dirname, '..', 'bin');
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }
  
  const binaryPath = path.join(binDir, BINARY_NAME);
  
  console.log(`📥 Downloading from: ${downloadUrl}`);
  
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(binaryPath);
    
    https.get(downloadUrl, (response) => {
      if (response.statusCode === 200) {
        response.pipe(file);
        file.on('finish', () => {
          file.close();
          
          // Make executable on Unix systems
          if (process.platform !== 'win32') {
            fs.chmodSync(binaryPath, '755');
          }
          
          console.log('✅ Shimmy installed successfully!');
          console.log(`📍 Binary location: ${binaryPath}`);
          console.log('🚀 Run "shimmy --help" to get started');
          resolve();
        });
      } else if (response.statusCode === 302 || response.statusCode === 301) {
        // Handle redirect
        https.get(response.headers.location, (redirectResponse) => {
          redirectResponse.pipe(file);
          file.on('finish', () => {
            file.close();
            if (process.platform !== 'win32') {
              fs.chmodSync(binaryPath, '755');
            }
            console.log('✅ Shimmy installed successfully!');
            resolve();
          });
        }).on('error', reject);
      } else {
        reject(new Error(`Download failed: ${response.statusCode} ${response.statusMessage}`));
      }
    }).on('error', reject);
    
    file.on('error', reject);
  });
}

// Run installation
downloadBinary().catch(error => {
  console.error('❌ Installation failed:', error.message);
  console.error('💡 Try installing manually from: https://github.com/Michael-A-Kuykendall/shimmy/releases');
  process.exit(1);
});
