import Autocomplete from './controllers/autocomplete_controller.mjs'

const application = Stimulus.Application.start()

application.register('autocomplete', Autocomplete)
