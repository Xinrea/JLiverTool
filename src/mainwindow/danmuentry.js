function createDanmuEntry(special, medal, sender, content) {
  let danmuEntry = document.createElement('span')
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
  let danmuSender = document.createElement('span')
  danmuSender.className = 'sender'
  if (content) sender = sender + '：'
  danmuSender.innerText = sender
  danmuEntry.appendChild(danmuSender)
  if (content) {
    if (content.url) {
      let danmuContent = document.createElement('span')
      danmuContent.className = 'content emoji'
      danmuContent.style.backgroundImage = `url(${content.url})`
      h =
        parseInt(
          document.documentElement.style.getPropertyValue('--danmu-size')
        ) + 6
      w = (h / content.height) * content.width
      danmuContent.style.width = w + 'px'
      danmuEntry.appendChild(danmuContent)
    } else {
      let danmuContent = document.createElement('span')
      danmuContent.className = 'content'
      danmuContent.innerText = content
      danmuEntry.appendChild(danmuContent)
    }
  }
  danmuEntry.onclick = function () {
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
