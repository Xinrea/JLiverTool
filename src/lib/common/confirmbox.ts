// Create a confirm-box with message and callback
export function createConfirmBox(message: string, confirmCallback: () => void): HTMLDivElement {
  let confirmBox = document.createElement('div')
  confirmBox.className = 'confirm-box'
  let confirmBoxContent = document.createElement('div')
  confirmBoxContent.className = 'confirm-box-content'
  confirmBoxContent.innerText = message
  confirmBox.appendChild(confirmBoxContent)
  let confirmBoxBtn = document.createElement('div')
  confirmBoxBtn.className = 'confirm-box-btn'
  let confirmBoxBtnCancel = document.createElement('button')
  confirmBoxBtnCancel.className = 'confirm-box-btn-cancel'
  confirmBoxBtnCancel.innerText = '取消'
  confirmBoxBtnCancel.onclick = function () {
    confirmBox.remove()
  }
  confirmBoxBtn.appendChild(confirmBoxBtnCancel)
  let confirmBoxBtnConfirm = document.createElement('button')
  confirmBoxBtnConfirm.className = 'confirm-box-btn-confirm'
  confirmBoxBtnConfirm.innerText = '确认'
  confirmBoxBtnConfirm.onclick = function () {
    confirmBox.remove()
    confirmCallback()
  }
  confirmBoxBtn.appendChild(confirmBoxBtnConfirm)
  confirmBox.appendChild(confirmBoxBtn)
  return confirmBox
}
