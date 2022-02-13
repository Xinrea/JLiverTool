function createDanmuEntry(special, medal, sender, content) {
  let danmuEntry = document.createElement('div')
  if (special) {
    danmuEntry.className = 'danmu_entry special'
  } else {
    danmuEntry.className = 'danmu_entry'
  }
  if (medal) {
    danmuEntry.appendChild(
      createMedal(medal.guardLevel, medal.name, medal.level)
    )
  }
  let danmuSender = document.createElement('div')
  danmuSender.className = 'sender'
  if (content) sender = sender + '：'
  danmuSender.innerText = sender
  danmuEntry.appendChild(danmuSender)
  if (content) {
    let danmuContent = document.createElement('div')
    danmuContent.className = 'content'
    danmuContent.innerText = content
    danmuEntry.appendChild(danmuContent)
  }
  danmuEntry.onclick = function (e) {
    danmuEntry.classList.toggle('selected')
    if (danmuEntry.classList.contains('selected')) {
      if (appStatus.lastSelectedDanmu) {
        appStatus.lastSelectedDanmu.classList.remove('selected')
      }
      appStatus.lastSelectedDanmu = danmuEntry
    } else {
      appStatus.lastSelectedDanmu = null
    }
    appStatus.autoScroll = appStatus.lastSelectedDanmu == null
  }
  return danmuEntry
}

function createEnterEntry(medal, sender) {
  return createDanmuEntry(false, medal, sender + ' 进入直播间')
}

function createEffectEntry(content) {
  return createDanmuEntry(false, null, content)
}
