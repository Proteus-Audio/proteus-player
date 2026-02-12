export interface Track {
  title: string
  artist: string
  album: string
  albumImageUrl?: string
  duration: number
}

export interface LoadPayload {
  path: string
  duration: number
  window_label: string
}
