// <div class="medal">
//   <div class="medal_label">
//     <div class="medal_name">杏仁儿</div>
//   </div>
//   <div class="medal_level">17</div>
// </div>

function createMedal(guardLevel, medalName, medalLevel) {
  let medal = document.createElement('div')
  medal.className = 'medal'
  let medalLabel = document.createElement('div')
  medalLabel.className = 'medal_label'
  medalLabel.style.backgroundImage = `var(--medal-level-${Math.floor(
    (medalLevel - 1) / 4
  )})`
  if (guardLevel > 0) {
    medal.style.borderColor = `var(--guard-level-border-${guardLevel})`
    let medalGuard = document.createElement('div')
    medalGuard.className = 'medal_guard'
    medalGuard.style.backgroundImage = `var(--guard-level-${guardLevel})`
    medalLabel.appendChild(medalGuard)
  } else {
    medal.style.borderColor = `var(--medal-level-border-${Math.floor(
      (medalLevel - 1) / 4
    )})`
  }
  let medalNameDiv = document.createElement('div')
  medalNameDiv.className = 'medal_name'
  medalNameDiv.innerText = medalName
  let medalLevelDiv = document.createElement('div')
  medalLevelDiv.className = 'medal_level'
  medalLevelDiv.innerText = medalLevel
  medalLevelDiv.style.color = `var(--medal-level-border-${Math.floor(
    (medalLevel - 1) / 4
  )})`
  medalLabel.appendChild(medalNameDiv)
  medal.appendChild(medalLabel)
  medal.appendChild(medalLevelDiv)
  return medal
}
