<template>
  <div class="container">
    <div class="timeline">
      <div class="timestamp current-time">{{ formatTime(currentTime) }}</div>
      <el-slider v-model="currentTimeAsPercent" :show-tooltip="false" size="small" />
      <div class="timestamp total-time">{{ formatTime(duration) }}</div>
    </div>

    <div class="controls">
      <el-icon @click="reset" class="reset" :size="15" :color="'#9e9e9e'">
        <IconReset />
      </el-icon>
      <el-icon @click="playPause" class="play-pause" :size="30" :color="'#fff'">
        <IconPlay v-if="!playing" />
        <IconPause v-else />
      </el-icon>
      <el-icon @click="shuffle" class="reset" :size="15" :color="'#9e9e9e'">
        <IconShuffle />
      </el-icon>
    </div>

    <div class="volume-control">
      <div class="volume-status">
        <el-icon v-if="volume === 0" :size="12" :color="'#9e9e9e'">
          <IconVolume0 />
        </el-icon>
        <el-icon v-else-if="volume < 30" :size="12" :color="'#9e9e9e'">
          <IconVolume1 />
        </el-icon>
        <el-icon v-else :size="16" :color="'#9e9e9e'">
          <IconVolume3 />
        </el-icon>
      </div>

      <el-slider v-model="volumeAsPercent" :show-tooltip="false" size="small" />
    </div>
  </div>
</template>

<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { computed, onMounted, onUnmounted, ref } from 'vue'
// import { VideoPause, VideoPlay } from "@element-plus/icons-vue"
import IconPlay from './components/Icons/Play.vue'
import IconPause from './components/Icons/Pause.vue'
import IconVolume0 from './components/Icons/Volume0.vue'
import IconVolume1 from './components/Icons/Volume1.vue'
import IconVolume3 from './components/Icons/Volume3.vue'
import IconReset from './components/Icons/Reset.vue'
import IconShuffle from './components/Icons/Shuffle.vue'
import { invoke } from '@tauri-apps/api/core'
import { Options, listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { defaultMenu } from './utils/menu'
import { Menu } from '@tauri-apps/api/menu'
// import { appWindow } from '@tauri-apps/api/window'
// import Slider from "element-plus";

const volume = ref(1.0)
const playing = ref(false)
const currentTime = ref(0)
const duration = ref(null as number | null)
const unlisten = ref(null as (() => void) | null)
const endStatusLoop = ref(null as (() => void) | null)
const menu = ref<Menu | null>(null)

interface Status {
  duration: number | null
  playing: boolean
  time: number
  volume: number
}

const currentTimeAsPercent = computed({
  get: () => {
    if (duration.value === null) return 0
    const percent = currentTime.value / duration.value
    if (percent > 1) return 100
    return percent * 100
  },
  set: (value: number) => {
    const time = (duration.value || 0) * (value / 100)
    invoke('seek', { position: time })
    // if (duration.value === null) return
    // const time = duration.value * (value / 100)
    // currentTime.value = time
  },
})

const volumeAsPercent = computed({
  get: () => {
    return volume.value * 100
  },
  set: (value: number) => {
    const volume = value / 100
    invoke('set_volume', { volume })
  },
})

const formatTime = (time: number | null) => {
  if (time === null) return '00:00'

  const minutes = Math.floor(time / 60)
    .toString()
    .padStart(2, '0')
  const seconds = Math.round(time % 60)
    .toString()
    .padStart(2, '0')
  return `${minutes}:${seconds}`
}

const playPause = async () => {
  const status = await invoke('play_pause')
  switch (status) {
    case 'Playing':
      playing.value = true
      break
    case 'Paused':
      playing.value = false
      break
    default:
      break
  }
}

const reset = async () => {
  await invoke('stop')
}

const shuffle = async () => {
  await invoke('refresh_tracks')
}

const setStatus = (status: Status) => {
  duration.value = status.duration
  playing.value = status.playing
  currentTime.value = status.time
  volume.value = status.volume
}

const unlisteners = ref<(() => void)[]>([])

// onBeforeMount(async () => {
// })

onMounted(async () => {
  const currentWindow = getCurrentWindow()
  const newMenu = await defaultMenu()
  await newMenu.setAsAppMenu()
  menu.value = newMenu
  const emitTarget = {
    target: {
      kind: 'WebviewWindow',
      label: currentWindow.label,
    },
  } as Options

  console.log('set as app menu')

  unlisten.value = await listen(
    'LOAD_FILE',
    (event) => {
      console.log(event)
      duration.value = (event.payload as any).duration
      // console.log(event.payload);
      // duration.value = event.payload.duration;
    },
    emitTarget,
  )

  // const status = await invoke('get_status');
  // switch (status) {
  //   case 'Playing':
  //     playing.value = true;
  //     break;
  //   case 'Paused':
  //     playing.value = false;
  //     break;
  //   default:
  //     break;
  // }
  // const time = await invoke('get_time');
  // currentTime.value = time;

  const updateStatus = await listen(
    'UPDATE_STATUS',
    (event) => {
      const payload = event.payload as Status
      console.log(payload)
      setStatus(payload)
    },
    emitTarget,
  )
  unlisteners.value.push(updateStatus)

  const helloMessage = await listen(
    'HELLO',
    (event) => {
      const payload = event.payload as string
      console.log(payload)
    },
    emitTarget,
  )
  unlisteners.value.push(helloMessage)
})

onUnmounted(() => {
  if (unlisten.value) {
    unlisten.value()
  }

  if (endStatusLoop.value) {
    endStatusLoop.value()
  }

  unlisteners.value.forEach((unlistener) => unlistener())
})
</script>

<style lang="scss" scoped>
html {
  overflow: hidden !important;

  body {
    margin: 0 !important;
    overflow: hidden;
    height: 100vh;
    .container {
      font-family:
        Inter,
        ui-sans-serif,
        system-ui,
        -apple-system,
        BlinkMacSystemFont,
        'Segoe UI',
        Roboto,
        'Helvetica Neue',
        Arial,
        'Noto Sans',
        sans-serif,
        'Apple Color Emoji',
        'Segoe UI Emoji',
        'Segoe UI Symbol',
        'Noto Color Emoji';
      font-size: 1rem;
      font-weight: normal;
      color: #9e9e9e;
      max-height: 100px;
      height: 100px;
      overflow: hidden;
      position: relative;

      :deep(.el-slider) {
        .el-slider__runway {
          height: 4px;
          background-color: #797979;
          .el-slider__bar {
            height: 4px;
          }
        }
      }
      .timeline {
        display: grid;
        // grid-template-columns: 40px 1fr 40px;
        grid-template-columns: 30px 1fr 30px;
        align-items: center;
        justify-items: center;
        padding: 0 1em;
        gap: 1em;
        height: 40px;
        position: absolute;
        top: -0.2em;
        left: 0;
        width: calc(100% - 2em);
        // margin-top: -1em;

        :deep(.el-slider) {
          .el-slider__runway {
            .el-slider__bar {
              height: 4px;
              background-color: #5d5d5d;
            }
          }

          .el-slider__button-wrapper {
            opacity: 0;
            pointer-events: none !important;

            .el-slider__button {
              pointer-events: none !important;
            }
          }
        }
        .timestamp {
          // font-family: 'Silkscreen', monospace;
          font-size: 0.8em;
          display: flex;
          align-items: center;
          justify-content: center;
          // padding: 0.5em;
          width: 100%;
        }
      }

      .controls {
        display: grid;
        grid-template-columns: 10fr 1fr 10fr;
        align-items: center;
        justify-items: center;

        .reset {
          margin: auto;
          cursor: pointer;
          justify-self: right;
        }
        .play-pause {
          margin: auto;
          cursor: pointer;
        }
      }
      .volume-control {
        display: grid;
        grid-template-columns: 1fr 9fr;
        align-items: center;
        justify-items: left;
        padding: 0.2em 1em 0;
        height: 50px;
        position: absolute;
        bottom: -0.4em;
        left: 0;
        width: calc(100% - 2em);
        pointer-events: none;

        :deep(.el-slider) {
          pointer-events: auto;
          .el-slider__button {
            height: 10px;
            width: 10px;
            background-color: #c4c4c4;
            border-width: 0;
          }
        }

        .volume-status {
          display: flex;
          align-items: center;
          justify-content: left;
          // padding: 0.5em;
          width: 100%;
        }
      }
    }
  }
}
</style>
