/// PubMed Format tags.
///
/// See https://pubmed.ncbi.nlm.nih.gov/help/#pubmed-format
#[allow(unused)]
#[allow(clippy::upper_case_acronyms)]
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum PubmedTag {
    /// AB - Abstract: English language abstract taken directly from the published article
    Abstract,
    /// AD - Affiliation: Author or corporate author addresses
    Affiliation,
    /// AID - Article Identifier: Article ID values supplied by the publisher may include the pii (controlled publisher identifier), doi (digital object identifier), or book accession
    ArticleIdentifier,
    /// AU - Author: Authors
    Author,
    /// AUID - Author Identifier: Unique identifier associated with an author, corporate author, or investigator name
    AuthorIdentifier,
    /// BTI - Book Title: Book Title
    BookTitle,
    /// CI - Copyright Information: Copyright statement provided by the publisher
    CopyrightInformation,
    /// CIN - Comment In: Reference containing a comment about the article
    CommentIn,
    /// CN - Corporate Author: Corporate author or group names with authorship responsibility
    CorporateAuthor,
    /// COI - Conflict of Interest: Conflict of interest statement
    ConflictOfInterest,
    /// CON - Comment On: Reference upon which the article comments
    CommentOn,
    /// CP - Chapter: Book chapter
    Chapter,
    /// CRDT - Create Date: The date the citation record was first created
    CreateDate,
    /// CRF - Corrected and republished from: Final, correct version of an article
    CorrectedAndRepublishedFrom,
    /// CRI - Corrected and republished in: Original article that was republished in corrected form
    CorrectedAndRepublishedIn,
    /// CTDT - Contribution Date: Book contribution date
    ContributionDate,
    /// CTI - Collection Title: Collection Title
    CollectionTitle,
    /// DCOM - Completion Date: NLM internal processing completion date
    CompletionDate,
    /// DDIN - Dataset described in: Citation for the primary article resulting from a dataset
    DatasetDescribedIn,
    /// DRIN - Dataset use reported in: Citation for an article that uses a dataset from another scientific article
    DatasetUseReportedIn,
    /// DEP - Date of Electronic Publication: Electronic publication date
    DateOfElectronicPublication,
    /// DP - Publication Date: The date the article was published
    PublicationDate,
    /// DRDT - Date Revised: Book Revision Date
    DateRevised,
    /// ECF - Expression of Concern For: Reference containing an expression of concern for an article
    ExpressionOfConcernFor,
    /// ECI - Expression of Concern In: Cites the original article for which there is an expression of concern
    ExpressionOfConcernIn,
    /// EDAT - Entry Date: The date the citation was added to PubMed; the date is set to the publication date if added more than 1 year after the date published
    EntryDate,
    /// EFR - Erratum For: Cites the original article for which there is a published erratum; as of 2016, partial retractions are considered errata
    ErratumFor,
    /// EIN - Erratum In: Cites a published erratum to the article
    ErratumIn,
    /// ED - Editor: Book editors
    Editor,
    /// EN - Edition: Book edition
    Edition,
    /// FAU - Full Author Name: Full author names
    FullAuthorName,
    /// FED - Full Editor Name: Full editor names
    FullEditorName,
    /// FIR - Full Investigator Name: Full investigator or collaborator names
    FullInvestigatorName,
    /// FPS - Full Personal Name as Subject: Full Personal Name of the subject of the article
    FullPersonalNameAsSubject,
    /// GN - General Note: Supplemental or descriptive information related to the document
    GeneralNote,
    /// GR - Grants and Funding: Grant numbers, contract numbers, and intramural research identifiers associated with a publication
    GrantsAndFunding,
    /// GS - Gene Symbol: Abbreviated gene names (used 1991 through 1996)
    GeneSymbol,
    /// IP - Issue: The number of the issue, part, or supplement of the journal in which the article was published
    Issue,
    /// IR - Investigator: Investigator or collaborator
    Investigator,
    /// IRAD - Investigator Affiliation: Investigator or collaborator addresses
    InvestigatorAffiliation,
    /// IS - ISSN: International Standard Serial Number of the journal
    Issn,
    /// ISBN - ISBN: International Standard Book Number
    Isbn,
    /// JID - NLM Unique ID: Unique journal ID in the NLM catalog of books, journals, and audiovisuals
    NlmUniqueId,
    /// JT - Full Journal Title: Full journal title from NLM cataloging data
    FullJournalTitle,
    /// LA - Language: The language in which the article was published
    Language,
    /// LID - Location ID: The pii or doi that serves the role of pagination
    LocationId,
    /// LR - Modification Date: Citation last revision date
    ModificationDate,
    /// MH - MeSH Terms: NLM Medical Subject Headings (MeSH) controlled vocabulary
    MeshTerms,
    /// MHDA - MeSH Date: The date MeSH terms were added to the citation. The MeSH date is the same as the Entrez date until MeSH are added
    MeshDate,
    /// MID - Manuscript Identifier: Identifier assigned to an author manuscript submitted to the NIH Manuscript Submission System
    ManuscriptIdentifier,
    /// NM - Substance Name: Supplementary Concept Record (SCR) data
    SubstanceName,
    /// OAB - Other Abstract: Abstract supplied by an NLM collaborating organization
    OtherAbstract,
    /// OABL - Other Abstract Language: Language of an abstract available from the publisher
    OtherAbstractLanguage,
    /// OCI - Other Copyright Information: Copyright owner
    OtherCopyrightInformation,
    /// OID - Other ID: Identification numbers provided by organizations supplying citation data
    OtherId,
    /// ORI - Original Report In: Cites the original article associated with the patient summary
    OriginalReportIn,
    /// OT - Other Term: Non-MeSH subject terms (keywords) either assigned by an organization identified by the Other Term Owner, or generated by the author and submitted by the publisher
    OtherTerm,
    /// OTO - Other Term Owner: Organization that may have provided the Other Term data
    OtherTermOwner,
    /// OWN - Owner: Organization acronym that supplied citation data
    Owner,
    /// PB - Publisher: Publishers of Books &amp; Documents citations
    Publisher,
    /// PG - Pagination: The full pagination of the article
    Pagination,
    /// PHST - Publication History Status Date: Publisher supplied dates regarding the article publishing process
    PublicationHistoryStatusDate,
    /// PL - Place of Publication: Journal's (country only) or book’s place of publication
    PlaceOfPublication,
    /// PMC - PubMed Central Identifier: Unique identifier for the cited article in PubMed Central (PMC)
    PubmedCentralIdentifier,
    /// PMCR - PMC Release: Availability of PMC article
    PmcRelease,
    /// PMID - PubMed Unique Identifier: Unique number assigned to each PubMed citation
    PubmedUniqueIdentifier,
    /// PS - Personal Name as Subject: Individual is the subject of the article
    PersonalNameAsSubject,
    /// PST - Publication Status: Publication status
    PublicationStatus,
    /// PT - Publication Type: The type of material the article represents
    PublicationType,
    /// RF - Number of References: Number of bibliographic references for Review articles
    NumberOfReferences,
    /// RIN - Retraction In: Retraction of the article
    RetractionIn,
    /// RN - EC/RN Number: Includes chemical, protocol or disease terms. May also include a number assigned by the Enzyme Commission or by the Chemical Abstracts Service.
    EcRnNumber,
    /// ROF - Retraction Of: Article being retracted
    RetractionOf,
    /// RPF - Republished From: Article being cited has been republished or reprinted in either full or abridged form from another source
    RepublishedFrom,
    /// RPI - Republished In: Article being cited also appears in another source in either full or abridged form
    RepublishedIn,
    /// RRI - Retracted and Republished In: Final, republished version of an article
    RetractedAndRepublishedIn,
    /// RRF - Retracted and Republished From: Original article that was retracted and republished
    RetractedAndRepublishedFrom,
    /// SB - Subset: Journal or citation subset values representing specialized topics
    Subset,
    /// SFM - Space Flight Mission: NASA-supplied data space flight/mission name and/or number
    SpaceFlightMission,
    /// SI - Secondary Source ID: Identifies secondary source databanks and accession numbers of molecular sequences discussed in articles
    SecondarySourceId,
    /// SO - Source: Composite field containing bibliographic information
    Source,
    /// SPIN - Summary For Patients In: Cites a patient summary article
    SummaryForPatientsIn,
    /// STAT - Status Tag: Used for internal processing at NLM
    StatusTag,
    /// TA - Journal Title Abbreviation: Standard journal title abbreviation
    JournalTitleAbbreviation,
    /// TI - Title: The title of the article
    Title,
    /// TT - Transliterated Title: Title of the article originally published in a non-English language, in that language
    TransliteratedTitle,
    /// UIN - Update In: Update to the article
    UpdateIn,
    /// UOF - Update Of: The article being updated
    UpdateOf,
    /// VI - Volume: Volume number of the journal
    Volume,
    /// VTI - Volume Title: Book Volume Title
    VolumeTitle,
}

impl PubmedTag {
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "AB" => Some(Self::Abstract),
            "AD" => Some(Self::Affiliation),
            "AID" => Some(Self::ArticleIdentifier),
            "AU" => Some(Self::Author),
            "AUID" => Some(Self::AuthorIdentifier),
            "BTI" => Some(Self::BookTitle),
            "CI" => Some(Self::CopyrightInformation),
            "CIN" => Some(Self::CommentIn),
            "CN" => Some(Self::CorporateAuthor),
            "COI" => Some(Self::ConflictOfInterest),
            "CON" => Some(Self::CommentOn),
            "CP" => Some(Self::Chapter),
            "CRDT" => Some(Self::CreateDate),
            "CRF" => Some(Self::CorrectedAndRepublishedFrom),
            "CRI" => Some(Self::CorrectedAndRepublishedIn),
            "CTDT" => Some(Self::ContributionDate),
            "CTI" => Some(Self::CollectionTitle),
            "DCOM" => Some(Self::CompletionDate),
            "DDIN" => Some(Self::DatasetDescribedIn),
            "DRIN" => Some(Self::DatasetUseReportedIn),
            "DEP" => Some(Self::DateOfElectronicPublication),
            "DP" => Some(Self::PublicationDate),
            "DRDT" => Some(Self::DateRevised),
            "ECF" => Some(Self::ExpressionOfConcernFor),
            "ECI" => Some(Self::ExpressionOfConcernIn),
            "EDAT" => Some(Self::EntryDate),
            "EFR" => Some(Self::ErratumFor),
            "EIN" => Some(Self::ErratumIn),
            "ED" => Some(Self::Editor),
            "EN" => Some(Self::Edition),
            "FAU" => Some(Self::FullAuthorName),
            "FED" => Some(Self::FullEditorName),
            "FIR" => Some(Self::FullInvestigatorName),
            "FPS" => Some(Self::FullPersonalNameAsSubject),
            "GN" => Some(Self::GeneralNote),
            "GR" => Some(Self::GrantsAndFunding),
            "GS" => Some(Self::GeneSymbol),
            "IP" => Some(Self::Issue),
            "IR" => Some(Self::Investigator),
            "IRAD" => Some(Self::InvestigatorAffiliation),
            "IS" => Some(Self::Issn),
            "ISBN" => Some(Self::Isbn),
            "JID" => Some(Self::NlmUniqueId),
            "JT" => Some(Self::FullJournalTitle),
            "LA" => Some(Self::Language),
            "LID" => Some(Self::LocationId),
            "LR" => Some(Self::ModificationDate),
            "MH" => Some(Self::MeshTerms),
            "MHDA" => Some(Self::MeshDate),
            "MID" => Some(Self::ManuscriptIdentifier),
            "NM" => Some(Self::SubstanceName),
            "OAB" => Some(Self::OtherAbstract),
            "OABL" => Some(Self::OtherAbstractLanguage),
            "OCI" => Some(Self::OtherCopyrightInformation),
            "OID" => Some(Self::OtherId),
            "ORI" => Some(Self::OriginalReportIn),
            "OT" => Some(Self::OtherTerm),
            "OTO" => Some(Self::OtherTermOwner),
            "OWN" => Some(Self::Owner),
            "PB" => Some(Self::Publisher),
            "PG" => Some(Self::Pagination),
            "PHST" => Some(Self::PublicationHistoryStatusDate),
            "PL" => Some(Self::PlaceOfPublication),
            "PMC" => Some(Self::PubmedCentralIdentifier),
            "PMCR" => Some(Self::PmcRelease),
            "PMID" => Some(Self::PubmedUniqueIdentifier),
            "PS" => Some(Self::PersonalNameAsSubject),
            "PST" => Some(Self::PublicationStatus),
            "PT" => Some(Self::PublicationType),
            "RF" => Some(Self::NumberOfReferences),
            "RIN" => Some(Self::RetractionIn),
            "RN" => Some(Self::EcRnNumber),
            "ROF" => Some(Self::RetractionOf),
            "RPF" => Some(Self::RepublishedFrom),
            "RPI" => Some(Self::RepublishedIn),
            "RRI" => Some(Self::RetractedAndRepublishedIn),
            "RRF" => Some(Self::RetractedAndRepublishedFrom),
            "SB" => Some(Self::Subset),
            "SFM" => Some(Self::SpaceFlightMission),
            "SI" => Some(Self::SecondarySourceId),
            "SO" => Some(Self::Source),
            "SPIN" => Some(Self::SummaryForPatientsIn),
            "STAT" => Some(Self::StatusTag),
            "TA" => Some(Self::JournalTitleAbbreviation),
            "TI" => Some(Self::Title),
            "TT" => Some(Self::TransliteratedTitle),
            "UIN" => Some(Self::UpdateIn),
            "UOF" => Some(Self::UpdateOf),
            "VI" => Some(Self::Volume),
            "VTI" => Some(Self::VolumeTitle),
            _ => None,
        }
    }

    pub fn as_tag(&self) -> &'static str {
        match self {
            Self::Abstract => "AB",
            Self::Affiliation => "AD",
            Self::ArticleIdentifier => "AID",
            Self::Author => "AU",
            Self::AuthorIdentifier => "AUID",
            Self::BookTitle => "BTI",
            Self::CopyrightInformation => "CI",
            Self::CommentIn => "CIN",
            Self::CorporateAuthor => "CN",
            Self::ConflictOfInterest => "COI",
            Self::CommentOn => "CON",
            Self::Chapter => "CP",
            Self::CreateDate => "CRDT",
            Self::CorrectedAndRepublishedFrom => "CRF",
            Self::CorrectedAndRepublishedIn => "CRI",
            Self::ContributionDate => "CTDT",
            Self::CollectionTitle => "CTI",
            Self::CompletionDate => "DCOM",
            Self::DatasetDescribedIn => "DDIN",
            Self::DatasetUseReportedIn => "DRIN",
            Self::DateOfElectronicPublication => "DEP",
            Self::PublicationDate => "DP",
            Self::DateRevised => "DRDT",
            Self::ExpressionOfConcernFor => "ECF",
            Self::ExpressionOfConcernIn => "ECI",
            Self::EntryDate => "EDAT",
            Self::ErratumFor => "EFR",
            Self::ErratumIn => "EIN",
            Self::Editor => "ED",
            Self::Edition => "EN",
            Self::FullAuthorName => "FAU",
            Self::FullEditorName => "FED",
            Self::FullInvestigatorName => "FIR",
            Self::FullPersonalNameAsSubject => "FPS",
            Self::GeneralNote => "GN",
            Self::GrantsAndFunding => "GR",
            Self::GeneSymbol => "GS",
            Self::Issue => "IP",
            Self::Investigator => "IR",
            Self::InvestigatorAffiliation => "IRAD",
            Self::Issn => "IS",
            Self::Isbn => "ISBN",
            Self::NlmUniqueId => "JID",
            Self::FullJournalTitle => "JT",
            Self::Language => "LA",
            Self::LocationId => "LID",
            Self::ModificationDate => "LR",
            Self::MeshTerms => "MH",
            Self::MeshDate => "MHDA",
            Self::ManuscriptIdentifier => "MID",
            Self::SubstanceName => "NM",
            Self::OtherAbstract => "OAB",
            Self::OtherAbstractLanguage => "OABL",
            Self::OtherCopyrightInformation => "OCI",
            Self::OtherId => "OID",
            Self::OriginalReportIn => "ORI",
            Self::OtherTerm => "OT",
            Self::OtherTermOwner => "OTO",
            Self::Owner => "OWN",
            Self::Publisher => "PB",
            Self::Pagination => "PG",
            Self::PublicationHistoryStatusDate => "PHST",
            Self::PlaceOfPublication => "PL",
            Self::PubmedCentralIdentifier => "PMC",
            Self::PmcRelease => "PMCR",
            Self::PubmedUniqueIdentifier => "PMID",
            Self::PersonalNameAsSubject => "PS",
            Self::PublicationStatus => "PST",
            Self::PublicationType => "PT",
            Self::NumberOfReferences => "RF",
            Self::RetractionIn => "RIN",
            Self::EcRnNumber => "RN",
            Self::RetractionOf => "ROF",
            Self::RepublishedFrom => "RPF",
            Self::RepublishedIn => "RPI",
            Self::RetractedAndRepublishedIn => "RRI",
            Self::RetractedAndRepublishedFrom => "RRF",
            Self::Subset => "SB",
            Self::SpaceFlightMission => "SFM",
            Self::SecondarySourceId => "SI",
            Self::Source => "SO",
            Self::SummaryForPatientsIn => "SPIN",
            Self::StatusTag => "STAT",
            Self::JournalTitleAbbreviation => "TA",
            Self::Title => "TI",
            Self::TransliteratedTitle => "TT",
            Self::UpdateIn => "UIN",
            Self::UpdateOf => "UOF",
            Self::Volume => "VI",
            Self::VolumeTitle => "VTI",
        }
    }
}
