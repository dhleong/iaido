def Settings( **kwargs ):
  if kwargs[ 'language' ] == 'rust':
    return {
      'ls': {
        'cargo': {
          'loadOutDirsFromCheck': True,
        },
        'procMacro': {
          'enable': True,
        },
        'diagnostics': {
          'disabled': ["unresolved-proc-macro"],
        },
      }
    }
