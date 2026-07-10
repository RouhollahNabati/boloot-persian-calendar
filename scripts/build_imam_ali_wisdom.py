#!/usr/bin/env python3
"""Generate data/wisdom/imam_ali.json with 120 Imam Ali (AS) wisdom quotes."""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
INPUT_PATH = REPO_ROOT / "data" / "wisdom" / "imam_ali.json"
OUTPUT_PATH = REPO_ROOT / "data" / "wisdom" / "imam_ali.json"
TARGET_COUNT = 120
REQUIRED_KEYS = ("fa", "fa_af", "ps", "tg", "en")

ATTRIBUTION = {
    "fa": "امیرالمومنین علی علیه السلام",
    "fa_af": "امیرالمومنین علی علیه السلام",
    "ps": "امیرالمومنین علی علیه السلام",
    "tg": "Имом Али (а)",
    "en": "Imam Ali (AS)",
}

EXISTING_EN: dict[str, str] = {
    "انسان تا وقتی امید دارد، در عذاب نیست.": (
        "A person is not in torment as long as they have hope."
    ),
    "دوستی و دشمنیِ مردم را بر پایه کارهایشان بشناس، نه بر پایه سخنانشان.": (
        "Know people's friendship and enmity by their deeds, not their words."
    ),
    "صبر، کلید فرج است.": "Patience is the key to relief.",
    "بهترین سخن، سخنی است که حق را گوید و باطل را نه.": (
        "The best speech is that which speaks truth and not falsehood."
    ),
    "دنیا را گریزگاه گرفته‌اند و آخرت را می‌زیبد.": (
        "They have taken the world as a refuge and forgotten the hereafter."
    ),
    "علم بهترین زینت انسان است.": "Knowledge is the best adornment of a person.",
    "هر که از علمش بهره نبرد، علمش بر او حجت نمی‌شود.": (
        "Whoever does not benefit from their knowledge, "
        "that knowledge is not proof against them."
    ),
    "بزرگ‌ترین غنیمت، سلامتی است.": "The greatest bounty is wellbeing.",
    "خوش‌بختی در سلامتی و امنیت است.": (
        "Happiness lies in wellbeing and security."
    ),
    "کم‌گویی و بیش‌کردن، زینت مردم است.": (
        "Few words and much action are the adornment of people."
    ),
    "بهترین عمل، آن است که انسان امید نداشته باشد که کسی جز خدا او را ببیند.": (
        "The best deed is one where a person does not hope "
        "that anyone but God sees them."
    ),
    "هر که خود را بشناسد، پروردگارش را می‌شناسد.": (
        "Whoever knows themselves knows their Lord."
    ),
    "دشمنی با مردم، بیماری دل‌هاست.": (
        "Enmity with people is a sickness of hearts."
    ),
    "بهترین ثروت، عقل است.": "The best wealth is intellect.",
    "عقل، سرمایه است که با آن نمی‌توان بخرید.": (
        "Intellect is a treasure that cannot be bought."
    ),
    "بهترین زهد، پنهان داشتن زهد است.": (
        "The best asceticism is concealing one's asceticism."
    ),
    "هر که نصیحت‌پذیر باشد، راه رشد را می‌یابد.": (
        "Whoever accepts advice finds the path of growth."
    ),
    "سخاوت، ریشه بزرگواری است.": "Generosity is the root of nobility.",
    "بخشندگی، در و دریای خداست.": (
        "Forgiveness is the lake and sea of God."
    ),
    "هر که به یاد خدا باشد، دلش آرام می‌گیرد.": (
        "Whoever remembers God, their heart finds peace."
    ),
    "تقوا، زینت هر کار است.": "Piety is the adornment of every deed.",
    "بهترین یاری، یاری خداست.": "The best help is God's help.",
    "هر که از مردم بیاموزد، عاقل می‌شود.": (
        "Whoever learns from people becomes wise."
    ),
    "سکوت، حفظ عقل است.": "Silence is the preservation of intellect.",
    "هر که سخن خود را نگه دارد، عیب خود را پنهان کرده است.": (
        "Whoever holds their tongue has hidden their fault."
    ),
    "بهترین دوست، آن کسی است که تو را به خدا یاد دهد.": (
        "The best friend is one who reminds you of God."
    ),
    "هر که به اندازه کفایت بسنده کند، غنی است.": (
        "Whoever is content with what suffices is rich."
    ),
    "قناعت، ثروتی است که فنا نمی‌شود.": (
        "Contentment is wealth that does not perish."
    ),
    "هر که از گناه بترسد، در امان است.": (
        "Whoever fears sin is safe."
    ),
    "توبه، پاک‌کننده گناه است.": "Repentance cleanses sin.",
    "بهترین جهاد، مجاهده با نفس است.": (
        "The best jihad is struggle against the self."
    ),
    "هر که نفس خود را بشناسد، دشمن خود را شناخته است.": (
        "Whoever knows their self has known their enemy."
    ),
    "عدالت، اساس حکومت است.": "Justice is the foundation of governance.",
    "هر که عدالت کند، مردم را نگه می‌دارد.": (
        "Whoever is just retains the people."
    ),
    "بهترین سیاست، راستی با مردم است.": (
        "The best policy is honesty with people."
    ),
    "هر که به مردم دروغ گوید، ایمان ندارد.": (
        "Whoever lies to people has no faith."
    ),
}

NEW_QUOTES: list[dict[str, str]] = [
    {"fa": "ارزش هر کس آن است که در آن چه خوب انجام می‌دهد.", "fa_af": "ارزش هر کس آن است که در آن چه خوب انجام می‌دهد.", "ps": "د هر چا ارزښت هغه دی چې په هغه کې ښه ترسره کوي.", "tg": "Арзиши ҳар кас он аст, ки дар он чи некӣ мекунад.", "en": "The worth of each person is what they do well."},
    {"fa": "مردم دشمن آنچه نمی‌دانند.", "fa_af": "مردم دشمن آنچه نمی‌دانند.", "ps": "خلک د هغه څه دښمن دي چې نه پېژني.", "tg": "Мардум душмани он чизанд, ки намедонанд.", "en": "People are enemies of what they do not know."},
    {"fa": "هر که قدر خود را نشناسد، در هلاکت است.", "fa_af": "هر که قدر خود را نشناسد، در هلاکت است.", "ps": "هر څوک چې د خپل ارزښت نه پېژني، په هلاکت کې دی.", "tg": "Ҳар кӣ қадри худро нашиносад, дар ҳалокат аст.", "en": "Whoever does not know their worth is in ruin."},
    {"fa": "آنجا نباش که به تو نیازی ندارند.", "fa_af": "آنجا نباش که به تو نیازی ندارند.", "ps": "هلته مه اوسه چې ته ورته اړتیا نه لرې.", "tg": "Он ҷо набош, ки ба ту ниёз надоранд.", "en": "Do not be where you are not needed."},
    {"fa": "حکمت، گمشده مؤمن است.", "fa_af": "حکمت، گمشده مؤمن است.", "ps": "حکمت د مومن ورک شوی شت دی.", "tg": "Ҳикмат гумшудаи мӯмин аст.", "en": "Wisdom is the believer's lost property."},
    {"fa": "علم بهتر از مال است.", "fa_af": "علم بهتر از مال است.", "ps": "علم له مال څخه ښه دی.", "tg": "Илм аз мол беҳтар аст.", "en": "Knowledge is better than wealth."},
    {"fa": "دنیا زندان مؤمن است.", "fa_af": "دنیا زندان مؤمن است.", "ps": "نړۍ د مومن زندان دی.", "tg": "Дунё зиндони мӯмин аст.", "en": "The world is the believer's prison."},
    {"fa": "آخرت، بهشت کافر است.", "fa_af": "آخرت، بهشت کافر است.", "ps": "آخرت د کافر جنت دی.", "tg": "Охират биҳишти кофир аст.", "en": "The hereafter is the disbeliever's paradise."},
    {"fa": "دوست واقعی آن است که با تو راست بگوید.", "fa_af": "دوست واقعی آن است که با تو راست بگوید.", "ps": "ریښتینی ملګری هغه دی چې له تا سره رښتیا ووایي.", "tg": "Дӯсти ҳақиқӣ он аст, ки бо ту ростӣ мегӯяд.", "en": "A true friend is one who is honest with you."},
    {"fa": "مرگ، کفاره گناه است.", "fa_af": "مرگ، کفاره گناه است.", "ps": "مړینه د ګناه کفاره ده.", "tg": "Марг каффораи гуноҳ аст.", "en": "Death is expiation for sin."},
    {"fa": "بزرگ‌ترین فقر، فقر به دنیاست.", "fa_af": "بزرگ‌ترین فقر، فقر به دنیاست.", "ps": "تر ټولو لوی فقر د نړۍ فقر دی.", "tg": "Бузургтарин фақр фақри дунё аст.", "en": "The greatest poverty is poverty toward the world."},
    {"fa": "هر که از دنیا بگذرد، آخرت را می‌یابد.", "fa_af": "هر که از دنیا بگذرد، آخرت را می‌یابد.", "ps": "هر څوک چې له نړۍ تېر شي، آخرت مومي.", "tg": "Ҳар кӣ аз дунё гузарад, охиратро меёбад.", "en": "Whoever passes through the world finds the hereafter."},
    {"fa": "نیکی، زینت هر کار است.", "fa_af": "نیکی، زینت هر کار است.", "ps": "نیکي د هر کار زینت دی.", "tg": "Некӣ зебу зинати ҳар кор аст.", "en": "Goodness is the adornment of every deed."},
    {"fa": "هر که به مردم نیکی کند، به خود نیکی کرده است.", "fa_af": "هر که به مردم نیکی کند، به خود نیکی کرده است.", "ps": "هر څوک چې خلکو ته نیکي وکړي، ځان ته یې نیکي کړې.", "tg": "Ҳар кӣ ба мардум некӣ кунад, ба худ некӣ кардааст.", "en": "Whoever does good to people has done good to themselves."},
    {"fa": "بدی، زشت‌ترین چیز است.", "fa_af": "بدی، زشت‌ترین چیز است.", "ps": "بدي تر ټولو بد شی دی.", "tg": "Бадӣ зиштарин чиз аст.", "en": "Evil is the ugliest thing."},
    {"fa": "هر که از بدی بترسد، در امان است.", "fa_af": "هر که از بدی بترسد، در امان است.", "ps": "هر څوک چې له بدي ووېرېږي، په امنیت کې دی.", "tg": "Ҳар кӣ аз бадӣ битарсад, дар амният аст.", "en": "Whoever fears evil is safe."},
    {"fa": "راستی، نجات‌بخش است.", "fa_af": "راستی، نجات‌بخش است.", "ps": "رښتیا خلاصونکونکې ده.", "tg": "Ростӣ наҷотбахш аст.", "en": "Truth is salvation."},
    {"fa": "دروغ، ریشه هلاکت است.", "fa_af": "دروغ، ریشه هلاکت است.", "ps": "دروغ د هلاکت رېښه ده.", "tg": "Дурӯғ решаи ҳалокат аст.", "en": "Falsehood is the root of ruin."},
    {"fa": "هر که دروغ گوید، عقل خود را فروخته است.", "fa_af": "هر که دروغ گوید، عقل خود را فروخته است.", "ps": "هر څوک چې دروغ ووایي، خپل عقل یې پلورلی دی.", "tg": "Ҳар кӣ дурӯғ гӯяд, ақли худро фурӯхтааст.", "en": "Whoever lies has sold their intellect."},
    {"fa": "هر که امانت را حفظ کند، ایمان دارد.", "fa_af": "هر که امانت را حفظ کند، ایمان دارد.", "ps": "هر څوک چې امانت وساتي، ایمان لري.", "tg": "Ҳар кӣ аманатро нигоҳ дорад, имон дорад.", "en": "Whoever keeps a trust has faith."},
    {"fa": "خیانت، نشانه نفاق است.", "fa_af": "خیانت، نشانه نفاق است.", "ps": "خیانت د نفاق نښه ده.", "tg": "Хиёнат нишонаи нифоқ аст.", "en": "Betrayal is a sign of hypocrisy."},
    {"fa": "وفا، ریشه دوستی است.", "fa_af": "وفا، ریشه دوستی است.", "ps": "وفا د ملګرتیا رېښه ده.", "tg": "Вафо решаи дӯстӣ аст.", "en": "Loyalty is the root of friendship."},
    {"fa": "هر که به عهد وفا کند، مرد است.", "fa_af": "هر که به عهد وفا کند، مرد است.", "ps": "هر څوک چې په عهد وفادار وي، سړی دی.", "tg": "Ҳар кӣ ба аҳд вафо кунад, мард аст.", "en": "Whoever keeps their covenant is a man."},
    {"fa": "شجاعت، نیمی از ایمان است.", "fa_af": "شجاعت، نیمی از ایمان است.", "ps": "شجاعت د ایمان نیمه ده.", "tg": "Шуҷоъат нисфи имон аст.", "en": "Courage is half of faith."},
    {"fa": "هر که از حق بگذرد، ظالم است.", "fa_af": "هر که از حق بگذرد، ظالم است.", "ps": "هر څوک چې له حق تېر شي، ظالم دی.", "tg": "Ҳар кӣ аз ҳақ гузарад, золим аст.", "en": "Whoever transgresses truth is an oppressor."},
    {"fa": "ظلم، تاریک‌ترین شب است.", "fa_af": "ظلم، تاریک‌ترین شب است.", "ps": "ظلم تر ټولو تیاره شپه ده.", "tg": "Зулм ториктарин шаб аст.", "en": "Oppression is the darkest night."},
    {"fa": "هر که ظلم کند، خود را خوار کرده است.", "fa_af": "هر که ظلم کند، خود را خوار کرده است.", "ps": "هر څوک چې ظلم وکړي، ځان یې ذلیل کړی دی.", "tg": "Ҳар кӣ зулм кунад, худро хор кардааст.", "en": "Whoever oppresses has humiliated themselves."},
    {"fa": "عدل، نور حکومت است.", "fa_af": "عدل، نور حکومت است.", "ps": "عدل د حکومت رڼا ده.", "tg": "Адл нури ҳукумат аст.", "en": "Justice is the light of governance."},
    {"fa": "جور، ریشه فساد است.", "fa_af": "جور، ریشه فساد است.", "ps": "جور د فساد رېښه ده.", "tg": "Ҷавр решаи фасод аст.", "en": "Tyranny is the root of corruption."},
    {"fa": "هر که جور کند، خود را هلاک کرده است.", "fa_af": "هر که جور کند، خود را هلاک کرده است.", "ps": "هر څوک چې جور وکړي، ځان یې هلاک کړی دی.", "tg": "Ҳар кӣ ҷавр кунад, худро ҳалок кардааст.", "en": "Whoever commits tyranny has ruined themselves."},
    {"fa": "رحمت، زینت انسان است.", "fa_af": "رحمت، زینت انسان است.", "ps": "رحمت د انسان زینت دی.", "tg": "Раҳмат зебу зинати одам аст.", "en": "Mercy is the adornment of a person."},
    {"fa": "هر که رحم کند، مورد رحمت قرار می‌گیرد.", "fa_af": "هر که رحم کند، مورد رحمت قرار می‌گیرد.", "ps": "هر څوک چې رحم وکړي، رحم پکږي.", "tg": "Ҳар кӣ раҳм кунад, мавриди раҳм мегардад.", "en": "Whoever shows mercy receives mercy."},
    {"fa": "قساوت، بیماری دل است.", "fa_af": "قساوت، بیماری دل است.", "ps": "قساوت د زړه ناروغي ده.", "tg": "Қасоват бемории дил аст.", "en": "Hardness of heart is a sickness of the soul."},
    {"fa": "هر که دل سخت دارد، از رحمت دور است.", "fa_af": "هر که دل سخت دارد، از رحمت دور است.", "ps": "هر څوک چې سخت زړه لري، له رحمته لرې دی.", "tg": "Ҳар кӣ дили сахт дорад, аз раҳмат дур аст.", "en": "Whoever has a hard heart is far from mercy."},
    {"fa": "تواضع، زینت عالم است.", "fa_af": "تواضع، زینت عالم است.", "ps": "تواضع د عالم زینت دی.", "tg": "Тавозӯъ зебу зинати олим аст.", "en": "Humility is the adornment of a scholar."},
    {"fa": "هر که فروتن باشد، بلند مرتبه است.", "fa_af": "هر که فروتن باشد، بلند مرتبه است.", "ps": "هر څوک چې فروتن وي، لوړ مقام لري.", "tg": "Ҳар кӣ фурӯтан бошад, баландмартба аст.", "en": "Whoever is humble is of high rank."},
    {"fa": "تکبر، ریشه ذلت است.", "fa_af": "تکبر، ریشه ذلت است.", "ps": "تکبر د ذلت رېښه ده.", "tg": "Такаббур решаи залил аст.", "en": "Arrogance is the root of humiliation."},
    {"fa": "هر که تکبر کند، خود را خوار کرده است.", "fa_af": "هر که تکبر کند، خود را خوار کرده است.", "ps": "هر څوک چې تکبر وکړي، ځان یې ذلیل کړی دی.", "tg": "Ҳар кӣ такаббур кунад, худро хор кардааст.", "en": "Whoever is arrogant has humiliated themselves."},
    {"fa": "حلم، زینت بزرگان است.", "fa_af": "حلم، زینت بزرگان است.", "ps": "حلم د لویانو زینت دی.", "tg": "Ҳилм зебу зинати бузургон аст.", "en": "Forbearance is the adornment of the great."},
    {"fa": "هر که حلم کند، عاقل است.", "fa_af": "هر که حلم کند، عاقل است.", "ps": "هر څوک چې حلم وکړي، عاقل دی.", "tg": "Ҳар кӣ ҳилм кунад, оқил аст.", "en": "Whoever is forbearing is wise."},
    {"fa": "عجله، ریشه پشیمانی است.", "fa_af": "عجله، ریشه پشیمانی است.", "ps": "عجله د پښېمانۍ رېښه ده.", "tg": "Аҷила решаи пушаймонӣ аст.", "en": "Haste is the root of regret."},
    {"fa": "هر که عجله کند، پشیمان می‌شود.", "fa_af": "هر که عجله کند، پشیمان می‌شود.", "ps": "هر څوک چې عجله وکړي، پښیمانېږي.", "tg": "Ҳар кӣ аҷила кунад, пушаймон мешавад.", "en": "Whoever is hasty becomes regretful."},
    {"fa": "تدبیر، نصف عقل است.", "fa_af": "تدبیر، نصف عقل است.", "ps": "تدبیر د عقل نیمه ده.", "tg": "Тадбир нисфи ақл аст.", "en": "Prudence is half of intellect."},
    {"fa": "هر که تدبیر کند، موفق می‌شود.", "fa_af": "هر که تدبیر کند، موفق می‌شود.", "ps": "هر څوک چې تدبیر وکړي، بریالی کېږي.", "tg": "Ҳар кӣ тадбир кунад, муваффақ мешавад.", "en": "Whoever is prudent succeeds."},
    {"fa": "جهل، تاریک‌ترین شب است.", "fa_af": "جهل، تاریک‌ترین شب است.", "ps": "جهل تر ټولو تیاره شپه ده.", "tg": "Ҷаҳл ториктарин шаб аст.", "en": "Ignorance is the darkest night."},
    {"fa": "هر که جاهل باشد، در گمراهی است.", "fa_af": "هر که جاهل باشد، در گمراهی است.", "ps": "هر څوک چې جاهل وي، په گمراهۍ کې دی.", "tg": "Ҳар кӣ ҷоҳил бошад, дар гумроҳӣ аст.", "en": "Whoever is ignorant is astray."},
    {"fa": "علم، نور راه است.", "fa_af": "علم، نور راه است.", "ps": "علم د لارې رڼا ده.", "tg": "Илм нури роҳ аст.", "en": "Knowledge is the light of the path."},
    {"fa": "هر که علم بیاموزد، راه را می‌یابد.", "fa_af": "هر که علم بیاموزد، راه را می‌یابد.", "ps": "هر څوک چې علم زده کړي، لاره مومي.", "tg": "Ҳар кӣ илм биомӯзад, роҳро меёбад.", "en": "Whoever learns knowledge finds the way."},
    {"fa": "عمل، ثمره علم است.", "fa_af": "عمل، ثمره علم است.", "ps": "عمل د علم ثمره ده.", "tg": "Амал самари илм аст.", "en": "Action is the fruit of knowledge."},
    {"fa": "هر که علم را عمل کند، عاقل است.", "fa_af": "هر که علم را عمل کند، عاقل است.", "ps": "هر څوک چې علم عمل کړي، عاقل دی.", "tg": "Ҳар кӣ илмро амал кунад, оқил аст.", "en": "Whoever acts on knowledge is wise."},
    {"fa": "نماز، ستون دین است.", "fa_af": "نماز، ستون دین است.", "ps": "لمونځ د دین ستون دی.", "tg": "Намоз сутуни дин аст.", "en": "Prayer is the pillar of religion."},
    {"fa": "هر که نماز را حفظ کند، دین را حفظ کرده است.", "fa_af": "هر که نماز را حفظ کند، دین را حفظ کرده است.", "ps": "هر څوک چې لمونځ وساتي، دین یې ساتلی دی.", "tg": "Ҳар кӣ намозро нигоҳ дорад, динро нигоҳ доштааст.", "en": "Whoever preserves prayer has preserved religion."},
    {"fa": "روزه، سپر از آتش است.", "fa_af": "روزه، سپر از آتش است.", "ps": "روژه د اور سپر دی.", "tg": "Рӯза сипари оташ аст.", "en": "Fasting is a shield from the Fire."},
    {"fa": "هر که روزه بگیرد، تقوا می‌یابد.", "fa_af": "هر که روزه بگیرد، تقوا می‌یابد.", "ps": "هر څوک چې روژه ونیسي، تقوا مومي.", "tg": "Ҳар кӣ рӯза бигирад, тақво меёбад.", "en": "Whoever fasts gains piety."},
    {"fa": "زکات، پاک‌کننده مال است.", "fa_af": "زکات، پاک‌کننده مال است.", "ps": "زکات د مال پاکوونکی دی.", "tg": "Закот поккунандаи мол аст.", "en": "Charity purifies wealth."},
    {"fa": "هر که زکات دهد، مالش پاک می‌شود.", "fa_af": "هر که زکات دهد، مالش پاک می‌شود.", "ps": "هر څوک چې زکات ورکړي، مال یې پاکېږي.", "tg": "Ҳар кӣ закот диҳад, молаш пок мешавад.", "en": "Whoever gives charity purifies their wealth."},
    {"fa": "حج، پرداخت قرض است.", "fa_af": "حج، پرداخت قرض است.", "ps": "حج د قرض تادیه ده.", "tg": "Ҳаж пардохти қарз аст.", "en": "Pilgrimage is paying one's debt."},
    {"fa": "هر که حج کند، بدهی خود را پرداخته است.", "fa_af": "هر که حج کند، بدهی خود را پرداخته است.", "ps": "هر څوک چې حج وکړي، خپله پور تادیه کړې.", "tg": "Ҳар кӣ ҳаж кунад, қарзи худро пардохтааст.", "en": "Whoever performs pilgrimage has paid their debt."},
    {"fa": "جهاد، راه نجات است.", "fa_af": "جهاد، راه نجات است.", "ps": "جهاد د خلاصون لاره ده.", "tg": "Ҷиҳод роҳи наҷот аст.", "en": "Struggle is the path of salvation."},
    {"fa": "هر که جهاد کند، در راه خداست.", "fa_af": "هر که جهاد کند، در راه خداست.", "ps": "هر څوک چې جهاد وکړي، په د خدای لاره کې دی.", "tg": "Ҳар кӣ ҷиҳод кунад, дар роҳи Худо аст.", "en": "Whoever struggles is on God's path."},
    {"fa": "امر به معروف، نور جامعه است.", "fa_af": "امر به معروف، نور جامعه است.", "ps": "د معروف امر د ټولنې رڼا ده.", "tg": "Амри маъруф нури ҷомеа аст.", "en": "Enjoining good is the light of society."},
    {"fa": "نهی از منکر، سپر از فساد است.", "fa_af": "نهی از منکر، سپر از فساد است.", "ps": "د منکر نهي د فساد سپر دی.", "tg": "Наҳи аз мункар сипари фасод аст.", "en": "Forbidding wrong is a shield from corruption."},
    {"fa": "هر که معروف را امر کند، مؤمن است.", "fa_af": "هر که معروف را امر کند، مؤمن است.", "ps": "هر څوک چې معروف امر کړي، مومن دی.", "tg": "Ҳар кӣ маъруфро амр кунад, мӯмин аст.", "en": "Whoever enjoins good is a believer."},
    {"fa": "هر که منکر را نهی کند، عاقل است.", "fa_af": "هر که منکر را نهی کند، عاقل است.", "ps": "هر څوک چې منکر نهي کړي، عاقل دی.", "tg": "Ҳар кӣ мункарро наҳӣ кунад, оқил аст.", "en": "Whoever forbids wrong is wise."},
    {"fa": "برادری، ریشه وحدت است.", "fa_af": "برادری، ریشه وحدت است.", "ps": "وروري د یوالي رېښه ده.", "tg": "Бародарӣ решаи ваҳдат аст.", "en": "Brotherhood is the root of unity."},
    {"fa": "هر که با برادر خود مهربان باشد، مؤمن است.", "fa_af": "هر که با برادر خود مهربان باشد، مؤمن است.", "ps": "هر څوک چې له خپل ورور سره مهربان وي، مومن دی.", "tg": "Ҳар кӣ бо бародари худ меҳрубон бошад, мӯмин аст.", "en": "Whoever is kind to their brother is a believer."},
    {"fa": "قطع رحم، ریشه نفرین است.", "fa_af": "قطع رحم، ریشه نفرین است.", "ps": "د رحم قطع د لعنت رېښه ده.", "tg": "Қатъи раҳм решаи лаънат аст.", "en": "Severing kinship is the root of curse."},
    {"fa": "هر که رحم را وصل کند، برکت می‌یابد.", "fa_af": "هر که رحم را وصل کند، برکت می‌یابد.", "ps": "هر څوک چې رحم وصل کړي، برکت مومي.", "tg": "Ҳар кӣ раҳмро васл кунад, баркат меёбад.", "en": "Whoever maintains kinship finds blessing."},
    {"fa": "احسان، زینت انسان است.", "fa_af": "احسان، زینت انسان است.", "ps": "احسان د انسان زینت دی.", "tg": "Иҳсон зебу зинати одам аст.", "en": "Excellence in kindness is the adornment of a person."},
    {"fa": "هر که احسان کند، مورد احسان قرار می‌گیرد.", "fa_af": "هر که احسان کند، مورد احسان قرار می‌گیرد.", "ps": "هر څوک چې احسان وکړي، احسان پکږي.", "tg": "Ҳар кӣ иҳсон кунад, мавриди иҳсон мегардад.", "en": "Whoever shows kindness receives kindness."},
    {"fa": "بخل، ریشه فقر است.", "fa_af": "بخل، ریشه فقر است.", "ps": "بخل د فقر رېښه ده.", "tg": "Бахл решаи фақр аст.", "en": "Stinginess is the root of poverty."},
    {"fa": "هر که بخل کند، خود را فقیر کرده است.", "fa_af": "هر که بخل کند، خود را فقیر کرده است.", "ps": "هر څوک چې بخل وکړي، ځان یې فقیر کړی دی.", "tg": "Ҳар кӣ бахл кунад, худро фақир кардааст.", "en": "Whoever is stingy has made themselves poor."},
    {"fa": "سخاوت، ریشه غنا است.", "fa_af": "سخاوت، ریشه غنا است.", "ps": "سخاوت د بډایۍ رېښه ده.", "tg": "Саховат решаи сарват аст.", "en": "Generosity is the root of wealth."},
    {"fa": "هر که سخاوت کند، غنی می‌شود.", "fa_af": "هر که سخاوت کند، غنی می‌شود.", "ps": "هر څوک چې سخاوت وکړي، بډای کېږي.", "tg": "Ҳар кӣ саховат кунад, сарватманд мешавад.", "en": "Whoever is generous becomes rich."},
    {"fa": "شکر، زینت نعمت است.", "fa_af": "شکر، زینت نعمت است.", "ps": "شکر د نعمت زینت دی.", "tg": "Шукр зебу зинати неъмат аст.", "en": "Gratitude is the adornment of bounty."},
    {"fa": "هر که شکر کند، نعمتش زیاد می‌شود.", "fa_af": "هر که شکر کند، نعمتش زیاد می‌شود.", "ps": "هر څوک چې شکر وکړي، نعمت یې ډېرېږي.", "tg": "Ҳар кӣ шукр кунад, неъматаш зиёд мешавад.", "en": "Whoever is grateful, their bounty increases."},
    {"fa": "کفران، ریشه زوال است.", "fa_af": "کفران، ریشه زوال است.", "ps": "کفران د زوال رېښه ده.", "tg": "Куфрон решаи завол аст.", "en": "Ingratitude is the root of decline."},
    {"fa": "هر که کفران کند، نعمتش می‌رود.", "fa_af": "هر که کفران کند، نعمتش می‌رود.", "ps": "هر څوک چې کفران وکړي، نعمت یې ورکېږي.", "tg": "Ҳар кӣ куфрон кунад, неъматаш меравад.", "en": "Whoever is ungrateful loses their bounty."},
    {"fa": "یأس، ریشه هلاکت است.", "fa_af": "یأس، ریشه هلاکت است.", "ps": "یأس د هلاکت رېښه ده.", "tg": "Яъс решаи ҳалокат аст.", "en": "Despair is the root of ruin."},
    {"fa": "هر که یأس کند، در گمراهی است.", "fa_af": "هر که یأس کند، در گمراهی است.", "ps": "هر څوک چې یأس وکړي، په گمراهۍ کې دی.", "tg": "Ҳар кӣ яъс кунад, дар гумроҳӣ аст.", "en": "Whoever despairs is astray."},
    {"fa": "امید، نور دل است.", "fa_af": "امید، نور دل است.", "ps": "امید د زړه رڼا ده.", "tg": "Умед нури дил аст.", "en": "Hope is the light of the heart."},
    {"fa": "هر که امید داشته باشد، در عذاب نیست.", "fa_af": "هر که امید داشته باشد، در عذاب نیست.", "ps": "هر څوک چې امید ولري، په عذاب کې نه دی.", "tg": "Ҳар кӣ умед дошта бошад, дар азоб нест.", "en": "Whoever has hope is not in torment."},
    {"fa": "توکل، راه نجات است.", "fa_af": "توکل، راه نجات است.", "ps": "توکل د خلاصون لاره ده.", "tg": "Таваккул роҳи наҷот аст.", "en": "Trust in God is the path of salvation."},
    {"fa": "هر که توکل کند، کافی است.", "fa_af": "هر که توکل کند، کافی است.", "ps": "هر څوک چې توکل وکړي، کافي دی.", "tg": "Ҳар кӣ таваккул кунад, кофӣ аст.", "en": "Whoever trusts in God, that is enough."},
]


def validate_quote(quote: dict[str, str], index: int) -> None:
    for key in REQUIRED_KEYS:
        value = quote.get(key, "")
        if not isinstance(value, str) or not value.strip():
            raise ValueError(f"Quote {index} missing or empty key: {key}")


def validate_attribution(attribution: dict[str, str]) -> None:
    for key in REQUIRED_KEYS:
        value = attribution.get(key, "")
        if not isinstance(value, str) or not value.strip():
            raise ValueError(f"Attribution missing or empty key: {key}")


def enrich_existing_quotes(existing: list[dict[str, str]]) -> list[dict[str, str]]:
    enriched: list[dict[str, str]] = []
    for quote in existing:
        fa = quote["fa"]
        en = EXISTING_EN.get(fa)
        if not en:
            raise KeyError(f"No English translation for existing quote: {fa!r}")
        enriched.append(
            {
                "fa": quote["fa"],
                "fa_af": quote.get("fa_af", quote["fa"]),
                "ps": quote["ps"],
                "tg": quote["tg"],
                "en": en,
            }
        )
    return enriched


def deduplicate_by_fa(quotes: list[dict[str, str]]) -> list[dict[str, str]]:
    seen: set[str] = set()
    unique: list[dict[str, str]] = []
    for quote in quotes:
        fa = quote["fa"]
        if fa in seen:
            continue
        seen.add(fa)
        unique.append(quote)
    return unique


def main() -> int:
    if not INPUT_PATH.is_file():
        print(f"Error: input file not found: {INPUT_PATH}", file=sys.stderr)
        return 1

    with INPUT_PATH.open(encoding="utf-8") as handle:
        data = json.load(handle)

    existing_quotes = data.get("quotes", [])
    if len(existing_quotes) != 36:
        print(
            f"Error: expected 36 existing quotes, found {len(existing_quotes)}",
            file=sys.stderr,
        )
        return 1

    if len(NEW_QUOTES) != 84:
        print(
            f"Error: expected 84 new quotes in script, found {len(NEW_QUOTES)}",
            file=sys.stderr,
        )
        return 1

    quotes = enrich_existing_quotes(existing_quotes) + NEW_QUOTES
    quotes = deduplicate_by_fa(quotes)

    if len(quotes) != TARGET_COUNT:
        print(
            f"Error: expected {TARGET_COUNT} quotes after merge, got {len(quotes)}",
            file=sys.stderr,
        )
        return 1

    for index, quote in enumerate(quotes):
        validate_quote(quote, index)

    validate_attribution(ATTRIBUTION)

    output = {"attribution": ATTRIBUTION, "quotes": quotes}

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_PATH.open("w", encoding="utf-8") as handle:
        json.dump(output, handle, ensure_ascii=False, indent=2)
        handle.write("\n")

    print(f"Generated {len(quotes)} quotes -> {OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
