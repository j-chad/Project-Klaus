import mongoose, {Document, Schema} from "mongoose";
import * as uuid from "uuid";

const defaultNames = [ // Taken from https://goodbyejohndoe.com/
    "Bartholomew Shoe",
    "Weir Doe",
    "Abraham Pigeon",
    "Gunther Beard",
    "Hermann P. Schnitzel",
    "Nigel Nigel",
    "Fig Nelson",
    "Gibson Montgomery-Gibson",
    "Caspian Bellevedere",
    "Lance Bogrol",
    "Gustav Purpleson",
    "Inverness McKenzie",
    "Dylan Meringue",
    "Archibald Northbottom",
    "Niles Peppertrout",
    "Brian Cumin",
    "Fleece Marigold",
    "Shequondolisa Bivouac",
    "Indigo Violet",
    "Natalya Undergrowth",
    "Wisteria Ravenclaw",
    "Rodney Artichoke",
    "Fletch Skinner",
    "Piff Jenkins",
    "Carnegie Mondover",
    "Valentino Morose",
    "Eric Widget",
    "Giles Posture",
    "Norman Gordon",
    "Gordon Norman",
    "Burgundy Flemming",
    "Girth Wiedenbauer",
    "Lurch Schpellchek",
    "Parsley Montana",
    "Fergus Douchebag",
    "Ursula Gurnmeister",
    "Bodrum Salvador",
    "Pelican Steve",
    "Gideon Guernsey-Marmaduke",
    "Druid Wensleydale",
    "Linguina Nettlewater",
    "Chaplain Mondover",
    "Jarvis Pepperspray",
    "Jonquil Von Haggerston",
    "Brandon Guidelines",
    "Sue Shei",
    "Ingredia Nutrisha",
    "Cecil Hipplington-Shoreditch",
    "Penny Tool",
    "Samuel Serif",
    "Manuel Internetiquette",
    "Eleanor Fant",
    "Nathaneal Down",
    "Hanson Deck",
    "Desmond Eagle",
    "Richard Tea",
    "Quiche Hollandaise",
    "Hans Down",
    "Will Barrow",
    "Guy Mann",
    "Phillip Anthropy",
    "Benjamin Evalent",
    "Sir Cumference",
    "Dianne Ameter",
    "Alan Fresco",
    "Spruce Springclean",
    "Chauffina Carr",
    "Max Conversion",
    "Malcolm Function",
    "Ruby Von Rails",
    "Jason Response",
    "Jake Weary",
    "Justin Case",
    "Douglas Lyphe",
    "Ruüd van Driver",
    "Theodore Handle",
    "Hilary Ouse",
    "Dominic L. Ement",
    "Hugh Saturation",
    "Jackson Pot",
    "Elon Gated",
    "Russell Sprout",
    "Jim Séchen",
    "Hugh Millie-Yate",
    "Joss Sticks",
    "Thomas R. Toe",
    "Miles Tone",
    "Ravi O'Leigh",
    "Barry Tone"
]

export interface IUserDocument extends Document {
    uuid: string;
    name: string;
}

export const UserSchema = new Schema({
    uuid: {
        type: String,
        index: true,
        unique: true,
        required: true,
        default: function () {
            return uuid.v4();
        }
    },
    name: {
        type: String,
        required: true,
        minlength: 4,
        maxlength: 30,
        default: function () {
            return defaultNames[Math.floor(Math.random() * defaultNames.length)];
        }
    }
});

const User = mongoose.model<IUserDocument>('User', UserSchema);

export default User;