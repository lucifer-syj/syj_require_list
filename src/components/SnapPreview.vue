<template>
  <div class="preview-box">
    <div class="preview-content">
      <div class="preview-icon">📍</div>
      <div class="preview-text">吸附预览</div>
    </div>
  </div>
</template>

<style scoped>
.preview-box {
  width: 100%;
  height: 100%;
  background-color: rgba(100, 150, 255, 0.25);
  border: 3px solid rgba(100, 150, 255, 0.6);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 0 20px rgba(100, 150, 255, 0.4);
}

.preview-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.preview-icon {
  font-size: 48px;
  opacity: 0.8;
  animation: pulse 2s ease-in-out infinite;
}

.preview-text {
  color: rgba(255, 255, 255, 0.9);
  font-size: 20px;
  font-weight: 600;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  letter-spacing: 2px;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 0.8;
  }
  50% {
    transform: scale(1.1);
    opacity: 1;
  }
}
</style>
