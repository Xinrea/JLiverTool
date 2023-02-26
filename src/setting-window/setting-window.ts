import Alpine from 'alpinejs'

Alpine.data('setting', ():any => {
  font_size: 14
})
Alpine.start()
setInterval(() => {
  location.reload();
}, 1000);