SELECT pgml.embed(transformer=> 'intfloat/e5-small',
                inputs => ARRAY['How do I get a replacement Medicare card?',
                                'What is the monthly premium for Medicare Part B?',
                                'How do I terminate my Medicare Part B (medical insurance)?',
                                'How do I sign up for Medicare?',
                                'How do I get a replacement Medicare card?'
                                ]);