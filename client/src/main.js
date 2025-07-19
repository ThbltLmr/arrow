const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let postureIcon;
let postureText;
let connectionIndicator;
let connectionMessage;
let historyList;

// Posture type mapping
const POSTURE_MAPPING = {
  'STRAIGHT': {
    icon: '/assets/good_posture.svg',
    message: 'Straight'
  },
  'SLOUCHING_BACK': {
    icon: '/assets/bad_posture.svg',
    message: 'Slouching back'
  },
  'LEANING_IN': {
    icon: '/assets/bad_posture.svg',
    message: 'Leaning in'
  },
  'HEAD_TILT_LEFT': {
    icon: '/assets/bad_posture.svg',
    message: 'Head tilt left'
  },
  'HEAD_TILT_RIGHT': {
    icon: '/assets/bad_posture.svg',
    message: 'Head tilt right'
  },
  'BODY_TILT_LEFT': {
    icon: '/assets/bad_posture.svg',
    message: 'Body tilt left'
  },
  'BODY_TILT_RIGHT': {
    icon: '/assets/bad_posture.svg',
    message: 'Body tilt right'
  },
  'SHOULDERS_NOT_VISIBLE': {
    icon: '/assets/bad_posture.svg',
    message: 'Shoulders not visible'
  },
  'HEAD_NOT_VISIBLE': {
    icon: '/assets/bad_posture.svg',
    message: 'Head not visible'
  },
  'UNKNOWN': {
    icon: '/assets/bad_posture.svg',
    message: 'Unknown'
  }
};

function updatePostureDisplay(posture, message) {
  const mapping = POSTURE_MAPPING[posture] || POSTURE_MAPPING['UNKNOWN'];
  
  postureIcon.src = mapping.icon;
  postureText.textContent = message || mapping.message;
  
  // Add visual feedback for posture changes
  postureIcon.style.transform = 'scale(0.9)';
  setTimeout(() => {
    postureIcon.style.transform = 'scale(1)';
  }, 200);
}

function updateConnectionStatus(connected, message) {
  connectionIndicator.className = connected ? 'status-connected' : 'status-disconnected';
  connectionMessage.textContent = message;
}

function formatDuration(durationMs) {
  const seconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  
  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  } else {
    return `${seconds}s`;
  }
}

async function loadSessionHistory() {
  try {
    const logs = await invoke('get_session_logs');
    updateHistoryDisplay(logs);
  } catch (error) {
    console.error('Failed to load session history:', error);
  }
}

function updateHistoryDisplay(logs) {
  if (!logs || logs.length === 0) {
    historyList.innerHTML = '<p class="no-history">No session data yet</p>';
    return;
  }
  
  const historyItems = logs.slice(0, 5).map(log => {
    const mapping = POSTURE_MAPPING[log.posture] || POSTURE_MAPPING['UNKNOWN'];
    return `
      <div class="history-item">
        <div class="posture-name">${mapping.message}</div>
        <div class="duration">${formatDuration(log.duration.secs * 1000 + log.duration.nanos / 1000000)}</div>
      </div>
    `;
  }).join('');
  
  historyList.innerHTML = historyItems;
}

async function initializeApp() {
  try {
    // Get initial posture state
    const currentPosture = await invoke('get_current_posture');
    updatePostureDisplay('UNKNOWN', currentPosture);
    
    // Load session history
    await loadSessionHistory();
    
    // Set up event listeners
    await listen('posture-changed', (event) => {
      const { posture, message } = event.payload;
      updatePostureDisplay(posture, message);
      
      // Reload history to show updated session data
      loadSessionHistory();
    });
    
    await listen('connection-status', (event) => {
      const { connected, message } = event.payload;
      updateConnectionStatus(connected, message);
    });
    
  } catch (error) {
    console.error('Failed to initialize app:', error);
    updateConnectionStatus(false, 'Failed to initialize');
  }
}

window.addEventListener("DOMContentLoaded", () => {
  postureIcon = document.querySelector("#posture-svg");
  postureText = document.querySelector("#posture-text");
  connectionIndicator = document.querySelector("#connection-indicator");
  connectionMessage = document.querySelector("#connection-message");
  historyList = document.querySelector("#history-list");
  
  // Add smooth transitions
  postureIcon.style.transition = 'transform 0.2s ease-in-out';
  
  initializeApp();
});