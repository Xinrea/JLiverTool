import { MedalInfo } from '../lib/types'

function cth(dColor: number) {
  return '#' + ('000000' + dColor.toString(16)).slice(-6)
}

export function createMedal(medal_info: MedalInfo): HTMLElement {
  let medal = document.createElement('div')
  medal.className = 'medal'
  medal.style.borderColor = `${cth(medal_info.medal_color_border)}`
  let medalLabel = document.createElement('div')
  medalLabel.className = 'medal_label'
  medalLabel.style.backgroundImage = `linear-gradient(45deg, ${cth(
    medal_info.medal_color_start
  )}, ${cth(medal_info.medal_color_end)})`
  if (medal_info.guard_level > 0) {
    let medalGuard = document.createElement('div')
    medalGuard.className = 'medal_guard'
    medalGuard.style.backgroundImage = `var(--guard-level-${medal_info.guard_level})`
    medalLabel.appendChild(medalGuard)
  }
  let medalNameDiv = document.createElement('div')
  medalNameDiv.className = 'medal_name'
  medalNameDiv.innerText = medal_info.medal_name
  let medalLevelDiv = document.createElement('div')
  medalLevelDiv.style.color = `${cth(medal_info.medal_color)}`
  medalLevelDiv.className = 'medal_level'
  medalLevelDiv.innerText = String(medal_info.medal_level)
  medalLabel.appendChild(medalNameDiv)
  medal.appendChild(medalLabel)
  medal.appendChild(medalLevelDiv)
  return medal
}
