pub fn generate_random_name() -> String {
    let adjectives = [
        "Abundant", "Acerbic", "Adamant", "Adroit", "Aesthetic", "Affable", "Ambivalent", "Amorphous", 
        "Arcane", "Archaic", "Astute", "Austere", "Belligerent", "Benevolent", "Blithe", "Boisterous", 
        "Callous", "Candid", "Capricious", "Caustic", "Cerebral", "Clandestine", "Cogent", "Complacent", 
        "Conciliatory", "Convivial", "Copious", "Cryptic", "Culpable", "Cursory", "Dauntless", "Decorous", 
        "Deft", "Deleterious", "Demure", "Derisive", "Despondent", "Didactic", "Diffident", "Diligent", 
        "Discreet", "Disparate", "Dogmatic", "Dormant", "Ebullient", "Eclectic", "Effervescent", "Efficacious", 
        "Effusive", "Egregious", "Elated", "Eloquent", "Elusive", "Enigmatic", "Ephemeral", "Esoteric", 
        "Ethereal", "Evanescent", "Exacerbated", "Exultant", "Facetious", "Fallacious", "Fastidious", "Fatuous", 
        "Fervent", "Flamboyant", "Flippant", "Fortuitous", "Frenetic", "Frivolous", "Frugal", "Furtive", 
        "Garrulous", "Glib", "Gregarious", "Hackneyed", "Haughty", "Heinous", "Hermetic", "Histrionic", 
        "Holistic", "Idiosyncratic", "Immutable", "Impetuous", "Implacable", "Inane", "Inchoate", "Incisive", 
        "Indefatigable", "Indigenous", "Indolent", "Ineffable", "Inert", "Ingenuous", "Inherent", "Innocuous", 
        "Insatiable", "Insidious", "Insipid", "Insolent", "Intransigent", "Intrepid", "Inveterate", "Irascible", 
        "Jaded", "Jocose", "Jovial", "Judicious", "Laconic", "Languid", "Latent", "Laudable", "Lethargic", 
        "Limpid", "Lithe", "Loquacious", "Lucid", "Luminous", "Magnanimous", "Malevolent", "Malleable", 
        "Manifest", "Melancholy", "Mercurial", "Meticulous", "Mirthful", "Mollified", "Morose", "Mundane", 
        "Nefarious", "Negligent", "Nonchalant", "Noxious", "Obdurate", "Obsequious", "Obstinate", "Obtuse", 
        "Ominous", "Opaque", "Ostentatious", "Pallid", "Pensive", "Perfunctory", "Pernicious", "Perspicacious", 
        "Petulant", "Phlegmatic", "Placid", "Platitudinous", "Plethoric", "Pragmatic", "Precarious", "Precocious", 
        "Prolific", "Puerile", "Pugnacious", "Punctilious", "Quixotic", "Recalcitrant", "Recondite", "Resilient", 
        "Reticent", "Rife", "Ruminative", "Sagacious", "Salient", "Sanguine", "Scrupulous", "Serene", 
        "Sinister", "Slovenly", "Solicitous", "Somber", "Spurious", "Stolid", "Stringent", "Succinct", 
        "Superfluous", "Surreptitious", "Taciturn", "Tenacious", "Tepid", "Terse", "Transient", "Trepidatious", 
        "Trite", "Ubiquitous", "Uncanny", "Unctuous", "Venerable", "Verbose", "Vexatious", "Vibrant", 
        "Vicarious", "Vigilant", "Visceral", "Vituperative", "Vivacious", "Volatile", "Voracious", "Winsome", 
        "Wistful", "Zealous", "Zenithal", "Hateful", "Zesty", "Freaky", "Suspicious", "Buoyant", "Loud", "Wide",
        "Delicious", "Rotund", "Ancient", "Masterwork", "Thin", "Conductive", "Oblong", "Sedentary", "Insufferable",
        "Annoying", "Viscous", "Shiny", "Cursed", "Doomed", "Cavernous", "Impervious"
    ];

    let nouns = [
        "Abdomen", "Achilles Tendon", "Adrenal Gland", "Alveoli", "Amygdala", "Ankle", "Aorta", "Appendix", 
        "Artery", "Atrium", "Auricle", "Back", "Bile Duct", "Bladder", "Blood Vessel", "Bone Marrow", 
        "Brain", "Brainstem", "Bronchi", "Brow", "Calf", "Capillary", "Cardiac Muscle", "Carotid Artery", 
        "Cartilage", "Cerebellum", "Cerebrum", "Cheek", "Chest", "Chin", "Clavicle", "Cochlea", 
        "Colon", "Conjunctiva", "Cornea", "Cranium", "Diaphragm", "Duodenum", "Ear", "Eardrum", 
        "Elbow", "Endocrine Gland", "Epidermis", "Epiglottis", "Esophagus", "Eye", "Eyebrow", "Eyelid", 
        "Femur", "Fibula", "Finger", "Fingernail", "Foot", "Forearm", "Forehead", "Gallbladder", 
        "Gingiva", "Gland", "Glottis", "Groin", "Gum", "Hair Follicle", "Hand", "Heart", 
        "Heel", "Hip", "Humerus", "Hypothalamus", "Ileum", "Incisor", "Index Finger", "Intestine", 
        "Iris", "Islets Of Langerhans", "Jaw", "Jejunum", "Joint", "Kidney", "Knee", "Knuckle", 
        "Labyrinth", "Lacrimal Gland", "Larynx", "Lens", "Ligament", "Lip", "Liver", "Lobe", 
        "Lumbar Vertebrae", "Lung", "Lymph Node", "Mandible", "Marrow", "Maxilla", "Medulla Oblongata", "Metacarpal", 
        "Midbrain", "Molar", "Muscle", "Nasal Cavity", "Neck", "Nerve", "Neuron", "Nose", 
        "Nostril", "Occipital Lobe", "Optic Nerve", "Ovary", "Palate", "Pancreas", "Pancreatic Duct", "Parathyroid Gland", 
        "Patella", "Pelvis", "Pericardium", "Phalanges", "Pharynx", "Pineal Gland", "Pituitary Gland", "Plasma", 
        "Platelet", "Pleura", "Pons", "Pore", "Prostate", "Pupil", "Pylorus", "Radius", 
        "Rectum", "Retina", "Rib", "Sacrum", "Salivary Gland", "Scalp", "Scapula", "Sclera", 
        "Sebaceous Gland", "Septum", "Shin", "Shoulder", "Sinus", "Skeleton", "Skull", "Small Intestine", 
        "Soft Palate", "Sole", "Spinal Cord", "Spleen", "Sternum", "Stomach", "Sweat Gland", "Talus", 
        "Tarsal", "Temple", "Tendon", "Testis", "Thalamus", "Thigh", "Thorax", "Throat", 
        "Thumb", "Thymus", "Thyroid", "Tibia", "Toe", "Tongue", "Tonsil", "Tooth", 
        "Trachea", "Trapezius", "Triceps", "Ulna", "Ureter", "Urethra", "Uvula", "Valve", 
        "Vein", "Ventricle", "Vertebra", "Vocal Cord", "Waist", "Wrist", "Zygomatic Bone",
    ];

    let adjective_index = rand::random::<u64>() % adjectives.len() as u64;
    let noun_index = rand::random::<u64>() % nouns.len() as u64;

    let adjective = adjectives[adjective_index as usize];
    let noun = nouns[noun_index as usize];

    format!("{} {}", adjective, noun)
}